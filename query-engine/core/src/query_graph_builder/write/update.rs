use super::*;
use crate::inputs::{RecordQueryFilterInput, UpdateRecordSelectorsInput};
use crate::query_graph_builder::write::limit::validate_limit;
use crate::query_graph_builder::write::write_args_parser::*;
use crate::{
    ArgumentListLookup, ParsedField, ParsedInputMap,
    query_ast::*,
    query_graph::{Node, NodeRef, QueryGraph, QueryGraphDependency},
};
use crate::{DataExpectation, ParsedObject, RowSink};
use psl::datamodel_connector::ConnectorCapability;
use query_structure::{Filter, Model};
use schema::{QuerySchema, constants::args};
use std::convert::TryInto;

/// Creates an update record query and adds it to the query graph, together with it's nested queries and companion read query.
pub(crate) fn update_record(
    graph: &mut QueryGraph,
    query_schema: &QuerySchema,
    model: Model,
    mut field: ParsedField<'_>,
) -> QueryGraphBuilderResult<()> {
    // "where"
    let where_arg: ParsedInputMap<'_> = field.arguments.lookup(args::WHERE).unwrap().value.try_into()?;
    let filter = extract_unique_filter(where_arg, &model)?;

    // "data"
    let data_argument = field.arguments.lookup(args::DATA).unwrap();
    let data_map: ParsedInputMap<'_> = data_argument.value.try_into()?;

    let can_use_atomic_update = can_use_atomic_update(query_schema, &model, &data_map, &field);

    let update_node = update_record_node(
        graph,
        query_schema,
        filter.clone(),
        model.clone(),
        data_map,
        Some(&field),
    )?;

    if !query_schema.has_capability(ConnectorCapability::UpdateReturning) || query_schema.relation_mode().is_prisma() {
        let read_parent_node = graph.create_node(utils::read_id_infallible(
            model.clone(),
            model.shard_aware_primary_identifier(),
            filter.clone(),
        ));

        if query_schema.relation_mode().is_prisma() {
            utils::insert_emulated_on_update(graph, query_schema, &model, &read_parent_node, &update_node)?;
        }

        graph.create_edge(
            &read_parent_node,
            &update_node,
            QueryGraphDependency::ProjectedDataSinkDependency(
                model.shard_aware_primary_identifier(),
                RowSink::All(&UpdateRecordSelectorsInput),
                Some(DataExpectation::non_empty_rows(
                    MissingRecord::builder().operation(DataOperation::Update).build(),
                )),
            ),
        )?;
    }

    // If the update can be done in a single operation (which includes getting the result back),
    // then make that node the result node.
    if can_use_atomic_update {
        graph.add_result_node(&update_node);

        let check_node = graph.create_node(Node::Empty);

        graph.create_edge(
            &update_node,
            &check_node,
            QueryGraphDependency::ProjectedDataDependency(
                model.shard_aware_primary_identifier(),
                Box::new(move |read_node, _| Ok(read_node)),
                Some(DataExpectation::non_empty_rows(
                    MissingRecord::builder().operation(DataOperation::Update).build(),
                )),
            ),
        )?;
    // Otherwise, perform the update, and then do an additional read.
    } else {
        graph.flag_transactional();

        let read_query = read::find_unique(field, model.clone(), query_schema)?;
        let read_node = graph.create_node(Query::Read(read_query));

        graph.add_result_node(&read_node);

        graph.create_edge(
            &update_node,
            &read_node,
            QueryGraphDependency::ProjectedDataSinkDependency(
                model.shard_aware_primary_identifier(),
                RowSink::ExactlyOneFilter(&RecordQueryFilterInput),
                Some(DataExpectation::non_empty_rows(
                    MissingRecord::builder().operation(DataOperation::Update).build(),
                )),
            ),
        )?;
    }

    Ok(())
}

/// Creates an update many record query and adds it to the query graph.
pub fn update_many_records(
    graph: &mut QueryGraph,
    query_schema: &QuerySchema,
    model: Model,
    with_field_selection: bool,
    mut field: ParsedField<'_>,
) -> QueryGraphBuilderResult<()> {
    graph.flag_transactional();

    // "where"
    let filter = match field.arguments.lookup(args::WHERE) {
        Some(where_arg) => extract_filter(where_arg.value.try_into()?, &model)?,
        None => Filter::empty(),
    };

    // "limit"
    let limit = validate_limit(field.arguments.lookup(args::LIMIT))?;

    // "data"
    let data_argument = field.arguments.lookup(args::DATA).unwrap();
    let data_map: ParsedInputMap<'_> = data_argument.value.try_into()?;

    if query_schema.relation_mode().uses_foreign_keys() {
        update_many_record_node(
            graph,
            query_schema,
            filter,
            model,
            data_map,
            UpdateManyRecordNodeOptionals {
                name: Some(field.name),
                nested_field_selection: field.nested_fields.filter(|_| with_field_selection),
                limit,
            },
        )?;
    } else {
        let pre_read_node = graph.create_node(utils::read_ids_infallible(
            model.clone(),
            model.shard_aware_primary_identifier(),
            filter,
        ));
        let update_many_node = update_many_record_node(
            graph,
            query_schema,
            Filter::empty(),
            model.clone(),
            data_map,
            UpdateManyRecordNodeOptionals {
                name: Some(field.name),
                nested_field_selection: field.nested_fields.filter(|_| with_field_selection),
                limit,
            },
        )?;

        utils::insert_emulated_on_update(graph, query_schema, &model, &pre_read_node, &update_many_node)?;

        graph.create_edge(
            &pre_read_node,
            &update_many_node,
            QueryGraphDependency::ProjectedDataDependency(
                model.shard_aware_primary_identifier(),
                Box::new(move |mut update_node, parent_ids| {
                    if let Node::Query(Query::Write(WriteQuery::UpdateManyRecords(ref mut ur))) = update_node {
                        ur.record_filter = parent_ids.into();
                    }

                    Ok(update_node)
                }),
                None,
            ),
        )?;
    }

    Ok(())
}

/// Creates an update record write node and adds it to the query graph.
pub fn update_record_node<T>(
    graph: &mut QueryGraph,
    query_schema: &QuerySchema,
    filter: T,
    model: Model,
    data_map: ParsedInputMap<'_>,
    field: Option<&ParsedField<'_>>,
) -> QueryGraphBuilderResult<NodeRef>
where
    T: Clone + Into<Filter>,
{
    let update_args = WriteArgsParser::from(&model, data_map)?;
    let mut args = update_args.args;

    args.update_datetimes(&model);

    let filter: Filter = filter.into();

    // If the connector can use `RETURNING`, always use it as it may:
    // 1. Save a SELECT statement
    // 2. Avoid from computing the ids in memory if they are updated. See `update_one_without_selection` function.
    let update_parent = if query_schema.has_capability(ConnectorCapability::UpdateReturning) {
        // If there's a selected field, fulfill the scalar selection set.
        if let Some(field) = field.cloned() {
            let nested_fields = field.nested_fields.unwrap().fields;
            let selection_order: Vec<String> = read::utils::collect_selection_order(&nested_fields);
            let selected_fields = read::utils::collect_selected_scalars(&nested_fields, &model);

            Query::Write(WriteQuery::UpdateRecord(UpdateRecord::WithSelection(
                UpdateRecordWithSelection {
                    name: field.name,
                    model: model.clone(),
                    record_filter: filter.into(),
                    args,
                    selected_fields,
                    selection_order,
                },
            )))
        // Otherwise, fallback to the primary identifier, that will be used to fulfill other nested operations requirements
        } else {
            let selected_fields = model.shard_aware_primary_identifier();
            let selection_order = selected_fields.db_names().collect();

            Query::Write(WriteQuery::UpdateRecord(UpdateRecord::WithSelection(
                UpdateRecordWithSelection {
                    name: String::new(), // This node will not be serialized so we don't need a name.
                    model: model.clone(),
                    record_filter: filter.into(),
                    args,
                    selected_fields,
                    selection_order,
                },
            )))
        }
    } else {
        Query::Write(WriteQuery::UpdateRecord(UpdateRecord::WithoutSelection(
            UpdateRecordWithoutSelection {
                model: model.clone(),
                record_filter: filter.into(),
                args,
            },
        )))
    };

    let update_node = graph.create_node(update_parent);

    for (relation_field, data_map) in update_args.nested {
        nested::connect_nested_query(graph, query_schema, update_node, relation_field, data_map)?;
    }

    Ok(update_node)
}

/// Creates an update many record query node and adds it to the query graph.
pub fn update_many_record_node<T>(
    graph: &mut QueryGraph,
    query_schema: &QuerySchema,
    filter: T,
    model: Model,
    data_map: ParsedInputMap<'_>,
    additional_args: UpdateManyRecordNodeOptionals<'_>,
) -> QueryGraphBuilderResult<NodeRef>
where
    T: Into<Filter>,
{
    graph.flag_transactional();

    let filter = filter.into();
    let record_filter = filter.into();
    let update_args = WriteArgsParser::from(&model, data_map)?;
    let mut args = update_args.args;

    args.update_datetimes(&model);

    let selected_fields = if let Some(nested_fields) = additional_args.nested_field_selection {
        let (selected_fields, selection_order, nested_read) =
            super::read::utils::extract_selected_fields(nested_fields.fields, &model, query_schema)?;

        Some(UpdateManyRecordsFields {
            fields: selected_fields,
            order: selection_order,
            nested: nested_read,
        })
    } else {
        None
    };

    let update_many = UpdateManyRecords {
        name: additional_args.name.unwrap_or_default(),
        model,
        record_filter,
        args,
        selected_fields,
        limit: additional_args.limit,
    };

    let update_many_node = graph.create_node(Query::Write(WriteQuery::UpdateManyRecords(update_many)));

    for (relation_field, data_map) in update_args.nested {
        nested::connect_nested_query(graph, query_schema, update_many_node, relation_field, data_map)?;
    }

    Ok(update_many_node)
}

/// An atomic update is an update performed in a single operation.
/// It uses `UPDATE ... RETURNING` when the connector supports it.
///
/// We only perform such update when:
/// 1. The connector supports returning updated values
/// 2. The selection set contains no relation
fn can_use_atomic_update(
    query_schema: &QuerySchema,
    model: &Model,
    data_map: &ParsedInputMap<'_>,
    field: &ParsedField<'_>,
) -> bool {
    // If the connector does not support RETURNING at all
    if !query_schema.has_capability(ConnectorCapability::UpdateReturning) {
        return false;
    }

    // If the operation has nested selection sets
    if field.has_nested_selection() {
        return false;
    }

    // If the operation has nested operations
    if WriteArgsParser::has_nested_operation(model, data_map) {
        return false;
    }

    true
}

pub struct UpdateManyRecordNodeOptionals<'a> {
    pub name: Option<String>,
    pub nested_field_selection: Option<ParsedObject<'a>>,
    pub limit: Option<usize>,
}
