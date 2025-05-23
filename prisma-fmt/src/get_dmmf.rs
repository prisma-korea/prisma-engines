use serde::Deserialize;

use crate::{schema_file_input::SchemaFileInput, validate};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GetDmmfParams {
    prisma_schema: SchemaFileInput,
    #[serde(default)]
    no_color: bool,
}

pub(crate) fn get_dmmf(params: &str) -> Result<String, String> {
    let params: GetDmmfParams = match serde_json::from_str(params) {
        Ok(params) => params,
        Err(serde_err) => {
            panic!("Failed to deserialize GetDmmfParams: {serde_err}");
        }
    };

    validate::run(params.prisma_schema, params.no_color).map(dmmf::dmmf_json_from_validated_schema)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use serde_json::json;

    #[test]
    fn get_dmmf_invalid_schema_with_colors() {
        let schema = r#"
            generator js {
            }

            datasøurce yolo {
            }
        "#;

        let request = json!({
            "prismaSchema": schema,
        });

        let expected = expect![[
            r#"{"error_code":"P1012","message":"\u001b[1;91merror\u001b[0m: \u001b[1mError validating: This line is invalid. It does not start with any known Prisma schema keyword.\u001b[0m\n  \u001b[1;94m-->\u001b[0m  \u001b[4mschema.prisma:5\u001b[0m\n\u001b[1;94m   | \u001b[0m\n\u001b[1;94m 4 | \u001b[0m\n\u001b[1;94m 5 | \u001b[0m            \u001b[1;91mdatasøurce yolo {\u001b[0m\n\u001b[1;94m 6 | \u001b[0m            }\n\u001b[1;94m   | \u001b[0m\n\u001b[1;91merror\u001b[0m: \u001b[1mError validating: This line is invalid. It does not start with any known Prisma schema keyword.\u001b[0m\n  \u001b[1;94m-->\u001b[0m  \u001b[4mschema.prisma:6\u001b[0m\n\u001b[1;94m   | \u001b[0m\n\u001b[1;94m 5 | \u001b[0m            datasøurce yolo {\n\u001b[1;94m 6 | \u001b[0m            \u001b[1;91m}\u001b[0m\n\u001b[1;94m 7 | \u001b[0m        \n\u001b[1;94m   | \u001b[0m\n\u001b[1;91merror\u001b[0m: \u001b[1mArgument \"provider\" is missing in generator block \"js\".\u001b[0m\n  \u001b[1;94m-->\u001b[0m  \u001b[4mschema.prisma:2\u001b[0m\n\u001b[1;94m   | \u001b[0m\n\u001b[1;94m 1 | \u001b[0m\n\u001b[1;94m 2 | \u001b[0m            \u001b[1;91mgenerator js {\u001b[0m\n\u001b[1;94m 3 | \u001b[0m            }\n\u001b[1;94m   | \u001b[0m\n\nValidation Error Count: 3"}"#
        ]];

        let response = get_dmmf(&request.to_string()).unwrap_err();
        expected.assert_eq(&response);
    }

    #[test]
    fn get_dmmf_missing_env_var() {
        let schema = r#"
            datasource thedb {
                provider = "postgresql"
                url = env("NON_EXISTING_ENV_VAR_WE_COUNT_ON_IT_AT_LEAST")
            }
        "#;

        let request = json!({
            "prismaSchema": schema,
        });

        let expected = expect![[r#"
            {
              "datamodel": {
                "enums": [],
                "models": [],
                "types": [],
                "indexes": []
              },
              "schema": {
                "inputObjectTypes": {},
                "outputObjectTypes": {
                  "prisma": [
                    {
                      "name": "Query",
                      "fields": []
                    },
                    {
                      "name": "Mutation",
                      "fields": [
                        {
                          "name": "executeRaw",
                          "args": [
                            {
                              "name": "query",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "String",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "parameters",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Json",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            }
                          ],
                          "isNullable": false,
                          "outputType": {
                            "type": "Json",
                            "location": "scalar",
                            "isList": false
                          }
                        },
                        {
                          "name": "queryRaw",
                          "args": [
                            {
                              "name": "query",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "String",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "parameters",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Json",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            }
                          ],
                          "isNullable": false,
                          "outputType": {
                            "type": "Json",
                            "location": "scalar",
                            "isList": false
                          }
                        }
                      ]
                    }
                  ]
                },
                "enumTypes": {
                  "prisma": [
                    {
                      "name": "TransactionIsolationLevel",
                      "values": [
                        "ReadUncommitted",
                        "ReadCommitted",
                        "RepeatableRead",
                        "Serializable"
                      ]
                    }
                  ]
                },
                "fieldRefTypes": {}
              },
              "mappings": {
                "modelOperations": [],
                "otherOperations": {
                  "read": [],
                  "write": [
                    "executeRaw",
                    "queryRaw"
                  ]
                }
              }
            }"#]];

        let response = get_dmmf(&request.to_string()).unwrap();

        let prettified_response =
            serde_json::to_string_pretty(&serde_json::from_str::<serde_json::Value>(&response).unwrap()).unwrap();

        expected.assert_eq(&prettified_response);
    }

    #[test]
    fn get_dmmf_direct_url_direct_empty() {
        let schema = r#"
            datasource thedb {
                provider = "postgresql"
                url = env("DBURL")
                directUrl = ""
            }
        "#;

        let request = json!({
            "prismaSchema": schema,
        });

        let expected = expect![[r#"
            {
              "datamodel": {
                "enums": [],
                "models": [],
                "types": [],
                "indexes": []
              },
              "schema": {
                "inputObjectTypes": {},
                "outputObjectTypes": {
                  "prisma": [
                    {
                      "name": "Query",
                      "fields": []
                    },
                    {
                      "name": "Mutation",
                      "fields": [
                        {
                          "name": "executeRaw",
                          "args": [
                            {
                              "name": "query",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "String",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "parameters",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Json",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            }
                          ],
                          "isNullable": false,
                          "outputType": {
                            "type": "Json",
                            "location": "scalar",
                            "isList": false
                          }
                        },
                        {
                          "name": "queryRaw",
                          "args": [
                            {
                              "name": "query",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "String",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "parameters",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Json",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            }
                          ],
                          "isNullable": false,
                          "outputType": {
                            "type": "Json",
                            "location": "scalar",
                            "isList": false
                          }
                        }
                      ]
                    }
                  ]
                },
                "enumTypes": {
                  "prisma": [
                    {
                      "name": "TransactionIsolationLevel",
                      "values": [
                        "ReadUncommitted",
                        "ReadCommitted",
                        "RepeatableRead",
                        "Serializable"
                      ]
                    }
                  ]
                },
                "fieldRefTypes": {}
              },
              "mappings": {
                "modelOperations": [],
                "otherOperations": {
                  "read": [],
                  "write": [
                    "executeRaw",
                    "queryRaw"
                  ]
                }
              }
            }"#]];

        let response = get_dmmf(&request.to_string()).unwrap();

        let prettified_response =
            serde_json::to_string_pretty(&serde_json::from_str::<serde_json::Value>(&response).unwrap()).unwrap();

        expected.assert_eq(&prettified_response);
    }

    #[test]
    fn get_dmmf_multiple_files() {
        let schema = vec![
            (
                "a.prisma",
                r#"
                datasource thedb {
                    provider = "postgresql"
                    url = env("DBURL")
                }

                model A {
                    id String @id
                    b_id String @unique
                    b B @relation(fields: [b_id], references: [id])
                }
            "#,
            ),
            (
                "b.prisma",
                r#"
                model B {
                    id String @id
                    a A?
                }
            "#,
            ),
        ];

        let request = json!({
            "prismaSchema": schema,
        });

        let expected = expect![[r#"
            {
              "datamodel": {
                "enums": [],
                "models": [
                  {
                    "name": "A",
                    "dbName": null,
                    "schema": null,
                    "fields": [
                      {
                        "name": "id",
                        "kind": "scalar",
                        "isList": false,
                        "isRequired": true,
                        "isUnique": false,
                        "isId": true,
                        "isReadOnly": false,
                        "hasDefaultValue": false,
                        "type": "String",
                        "nativeType": null,
                        "isGenerated": false,
                        "isUpdatedAt": false
                      },
                      {
                        "name": "b_id",
                        "kind": "scalar",
                        "isList": false,
                        "isRequired": true,
                        "isUnique": true,
                        "isId": false,
                        "isReadOnly": true,
                        "hasDefaultValue": false,
                        "type": "String",
                        "nativeType": null,
                        "isGenerated": false,
                        "isUpdatedAt": false
                      },
                      {
                        "name": "b",
                        "kind": "object",
                        "isList": false,
                        "isRequired": true,
                        "isUnique": false,
                        "isId": false,
                        "isReadOnly": false,
                        "hasDefaultValue": false,
                        "type": "B",
                        "nativeType": null,
                        "relationName": "AToB",
                        "relationFromFields": [
                          "b_id"
                        ],
                        "relationToFields": [
                          "id"
                        ],
                        "isGenerated": false,
                        "isUpdatedAt": false
                      }
                    ],
                    "primaryKey": null,
                    "uniqueFields": [],
                    "uniqueIndexes": [],
                    "isGenerated": false
                  },
                  {
                    "name": "B",
                    "dbName": null,
                    "schema": null,
                    "fields": [
                      {
                        "name": "id",
                        "kind": "scalar",
                        "isList": false,
                        "isRequired": true,
                        "isUnique": false,
                        "isId": true,
                        "isReadOnly": false,
                        "hasDefaultValue": false,
                        "type": "String",
                        "nativeType": null,
                        "isGenerated": false,
                        "isUpdatedAt": false
                      },
                      {
                        "name": "a",
                        "kind": "object",
                        "isList": false,
                        "isRequired": false,
                        "isUnique": false,
                        "isId": false,
                        "isReadOnly": false,
                        "hasDefaultValue": false,
                        "type": "A",
                        "nativeType": null,
                        "relationName": "AToB",
                        "relationFromFields": [],
                        "relationToFields": [],
                        "isGenerated": false,
                        "isUpdatedAt": false
                      }
                    ],
                    "primaryKey": null,
                    "uniqueFields": [],
                    "uniqueIndexes": [],
                    "isGenerated": false
                  }
                ],
                "types": [],
                "indexes": [
                  {
                    "model": "A",
                    "type": "id",
                    "isDefinedOnField": true,
                    "fields": [
                      {
                        "name": "id"
                      }
                    ]
                  },
                  {
                    "model": "A",
                    "type": "unique",
                    "isDefinedOnField": true,
                    "fields": [
                      {
                        "name": "b_id"
                      }
                    ]
                  },
                  {
                    "model": "B",
                    "type": "id",
                    "isDefinedOnField": true,
                    "fields": [
                      {
                        "name": "id"
                      }
                    ]
                  }
                ]
              },
              "schema": {
                "inputObjectTypes": {
                  "prisma": [
                    {
                      "name": "AWhereInput",
                      "meta": {
                        "source": "A",
                        "grouping": "A"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "AND",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "AWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "AWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": true
                            }
                          ]
                        },
                        {
                          "name": "OR",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "AWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": true
                            }
                          ]
                        },
                        {
                          "name": "NOT",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "AWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "AWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": true
                            }
                          ]
                        },
                        {
                          "name": "id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "StringFilter",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "b_id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "StringFilter",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "b",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BScalarRelationFilter",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "BWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "AOrderByWithRelationInput",
                      "meta": {
                        "grouping": "A"
                      },
                      "constraints": {
                        "maxNumFields": 1,
                        "minNumFields": 0
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "SortOrder",
                              "namespace": "prisma",
                              "location": "enumTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "b_id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "SortOrder",
                              "namespace": "prisma",
                              "location": "enumTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "b",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BOrderByWithRelationInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "AWhereUniqueInput",
                      "meta": {
                        "source": "A",
                        "grouping": "A"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": 1,
                        "fields": [
                          "id",
                          "b_id"
                        ]
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "b_id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "AND",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "AWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "AWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": true
                            }
                          ]
                        },
                        {
                          "name": "OR",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "AWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": true
                            }
                          ]
                        },
                        {
                          "name": "NOT",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "AWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "AWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": true
                            }
                          ]
                        },
                        {
                          "name": "b",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BScalarRelationFilter",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "BWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "AOrderByWithAggregationInput",
                      "meta": {
                        "grouping": "A"
                      },
                      "constraints": {
                        "maxNumFields": 1,
                        "minNumFields": 0
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "SortOrder",
                              "namespace": "prisma",
                              "location": "enumTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "b_id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "SortOrder",
                              "namespace": "prisma",
                              "location": "enumTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "_count",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "ACountOrderByAggregateInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "_max",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "AMaxOrderByAggregateInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "_min",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "AMinOrderByAggregateInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "AScalarWhereWithAggregatesInput",
                      "meta": {
                        "source": "A",
                        "grouping": "A"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "AND",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "AScalarWhereWithAggregatesInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "AScalarWhereWithAggregatesInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": true
                            }
                          ]
                        },
                        {
                          "name": "OR",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "AScalarWhereWithAggregatesInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": true
                            }
                          ]
                        },
                        {
                          "name": "NOT",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "AScalarWhereWithAggregatesInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "AScalarWhereWithAggregatesInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": true
                            }
                          ]
                        },
                        {
                          "name": "id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "StringWithAggregatesFilter",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "b_id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "StringWithAggregatesFilter",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "BWhereInput",
                      "meta": {
                        "source": "B",
                        "grouping": "B"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "AND",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "BWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": true
                            }
                          ]
                        },
                        {
                          "name": "OR",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": true
                            }
                          ]
                        },
                        {
                          "name": "NOT",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "BWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": true
                            }
                          ]
                        },
                        {
                          "name": "id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "StringFilter",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "a",
                          "isRequired": false,
                          "isNullable": true,
                          "inputTypes": [
                            {
                              "type": "ANullableScalarRelationFilter",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "AWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "Null",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "BOrderByWithRelationInput",
                      "meta": {
                        "grouping": "B"
                      },
                      "constraints": {
                        "maxNumFields": 1,
                        "minNumFields": 0
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "SortOrder",
                              "namespace": "prisma",
                              "location": "enumTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "a",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "AOrderByWithRelationInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "BWhereUniqueInput",
                      "meta": {
                        "source": "B",
                        "grouping": "B"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": 1,
                        "fields": [
                          "id"
                        ]
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "AND",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "BWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": true
                            }
                          ]
                        },
                        {
                          "name": "OR",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": true
                            }
                          ]
                        },
                        {
                          "name": "NOT",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "BWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": true
                            }
                          ]
                        },
                        {
                          "name": "a",
                          "isRequired": false,
                          "isNullable": true,
                          "inputTypes": [
                            {
                              "type": "ANullableScalarRelationFilter",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "AWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "Null",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "BOrderByWithAggregationInput",
                      "meta": {
                        "grouping": "B"
                      },
                      "constraints": {
                        "maxNumFields": 1,
                        "minNumFields": 0
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "SortOrder",
                              "namespace": "prisma",
                              "location": "enumTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "_count",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BCountOrderByAggregateInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "_max",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BMaxOrderByAggregateInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "_min",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BMinOrderByAggregateInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "BScalarWhereWithAggregatesInput",
                      "meta": {
                        "source": "B",
                        "grouping": "B"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "AND",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BScalarWhereWithAggregatesInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "BScalarWhereWithAggregatesInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": true
                            }
                          ]
                        },
                        {
                          "name": "OR",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BScalarWhereWithAggregatesInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": true
                            }
                          ]
                        },
                        {
                          "name": "NOT",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BScalarWhereWithAggregatesInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "BScalarWhereWithAggregatesInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": true
                            }
                          ]
                        },
                        {
                          "name": "id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "StringWithAggregatesFilter",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "ACreateInput",
                      "meta": {
                        "grouping": "A"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": true,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "b",
                          "isRequired": true,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BCreateNestedOneWithoutAInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "AUncheckedCreateInput",
                      "meta": {
                        "grouping": "A"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": true,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "b_id",
                          "isRequired": true,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "AUpdateInput",
                      "meta": {
                        "grouping": "A"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldUpdateOperationsInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "b",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BUpdateOneRequiredWithoutANestedInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "AUncheckedUpdateInput",
                      "meta": {
                        "grouping": "A"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldUpdateOperationsInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "b_id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldUpdateOperationsInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "ACreateManyInput",
                      "meta": {
                        "grouping": "A"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": true,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "b_id",
                          "isRequired": true,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "AUpdateManyMutationInput",
                      "meta": {
                        "grouping": "A"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldUpdateOperationsInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "AUncheckedUpdateManyInput",
                      "meta": {
                        "grouping": "A"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldUpdateOperationsInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "b_id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldUpdateOperationsInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "BCreateInput",
                      "meta": {
                        "grouping": "B"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": true,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "a",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "ACreateNestedOneWithoutBInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "BUncheckedCreateInput",
                      "meta": {
                        "grouping": "B"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": true,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "a",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "AUncheckedCreateNestedOneWithoutBInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "BUpdateInput",
                      "meta": {
                        "grouping": "B"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldUpdateOperationsInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "a",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "AUpdateOneWithoutBNestedInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "BUncheckedUpdateInput",
                      "meta": {
                        "grouping": "B"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldUpdateOperationsInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "a",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "AUncheckedUpdateOneWithoutBNestedInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "BCreateManyInput",
                      "meta": {
                        "grouping": "B"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": true,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "BUpdateManyMutationInput",
                      "meta": {
                        "grouping": "B"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldUpdateOperationsInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "BUncheckedUpdateManyInput",
                      "meta": {
                        "grouping": "B"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldUpdateOperationsInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "StringFilter",
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "equals",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "in",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": true
                            },
                            {
                              "type": "ListStringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "notIn",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": true
                            },
                            {
                              "type": "ListStringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "lt",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "lte",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "gt",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "gte",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "contains",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "startsWith",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "endsWith",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "mode",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "QueryMode",
                              "namespace": "prisma",
                              "location": "enumTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "not",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "NestedStringFilter",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "BScalarRelationFilter",
                      "meta": {
                        "grouping": "B"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "is",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "isNot",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "ACountOrderByAggregateInput",
                      "meta": {
                        "grouping": "A"
                      },
                      "constraints": {
                        "maxNumFields": 1,
                        "minNumFields": 1
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "SortOrder",
                              "namespace": "prisma",
                              "location": "enumTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "b_id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "SortOrder",
                              "namespace": "prisma",
                              "location": "enumTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "AMaxOrderByAggregateInput",
                      "meta": {
                        "grouping": "A"
                      },
                      "constraints": {
                        "maxNumFields": 1,
                        "minNumFields": 1
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "SortOrder",
                              "namespace": "prisma",
                              "location": "enumTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "b_id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "SortOrder",
                              "namespace": "prisma",
                              "location": "enumTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "AMinOrderByAggregateInput",
                      "meta": {
                        "grouping": "A"
                      },
                      "constraints": {
                        "maxNumFields": 1,
                        "minNumFields": 1
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "SortOrder",
                              "namespace": "prisma",
                              "location": "enumTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "b_id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "SortOrder",
                              "namespace": "prisma",
                              "location": "enumTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "StringWithAggregatesFilter",
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "equals",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "in",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": true
                            },
                            {
                              "type": "ListStringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "notIn",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": true
                            },
                            {
                              "type": "ListStringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "lt",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "lte",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "gt",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "gte",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "contains",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "startsWith",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "endsWith",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "mode",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "QueryMode",
                              "namespace": "prisma",
                              "location": "enumTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "not",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "NestedStringWithAggregatesFilter",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "_count",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "NestedIntFilter",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "_min",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "NestedStringFilter",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "_max",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "NestedStringFilter",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "ANullableScalarRelationFilter",
                      "meta": {
                        "grouping": "A"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "is",
                          "isRequired": false,
                          "isNullable": true,
                          "inputTypes": [
                            {
                              "type": "AWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "Null",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "isNot",
                          "isRequired": false,
                          "isNullable": true,
                          "inputTypes": [
                            {
                              "type": "AWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "Null",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "BCountOrderByAggregateInput",
                      "meta": {
                        "grouping": "B"
                      },
                      "constraints": {
                        "maxNumFields": 1,
                        "minNumFields": 1
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "SortOrder",
                              "namespace": "prisma",
                              "location": "enumTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "BMaxOrderByAggregateInput",
                      "meta": {
                        "grouping": "B"
                      },
                      "constraints": {
                        "maxNumFields": 1,
                        "minNumFields": 1
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "SortOrder",
                              "namespace": "prisma",
                              "location": "enumTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "BMinOrderByAggregateInput",
                      "meta": {
                        "grouping": "B"
                      },
                      "constraints": {
                        "maxNumFields": 1,
                        "minNumFields": 1
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "SortOrder",
                              "namespace": "prisma",
                              "location": "enumTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "BCreateNestedOneWithoutAInput",
                      "meta": {
                        "grouping": "B"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "create",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BCreateWithoutAInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "BUncheckedCreateWithoutAInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "connectOrCreate",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BCreateOrConnectWithoutAInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "connect",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BWhereUniqueInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "StringFieldUpdateOperationsInput",
                      "meta": {
                        "grouping": "A"
                      },
                      "constraints": {
                        "maxNumFields": 1,
                        "minNumFields": 1
                      },
                      "fields": [
                        {
                          "name": "set",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "BUpdateOneRequiredWithoutANestedInput",
                      "meta": {
                        "grouping": "B"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "create",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BCreateWithoutAInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "BUncheckedCreateWithoutAInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "connectOrCreate",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BCreateOrConnectWithoutAInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "upsert",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BUpsertWithoutAInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "connect",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BWhereUniqueInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "update",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BUpdateToOneWithWhereWithoutAInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "BUpdateWithoutAInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "BUncheckedUpdateWithoutAInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "ACreateNestedOneWithoutBInput",
                      "meta": {
                        "grouping": "A"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "create",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "ACreateWithoutBInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "AUncheckedCreateWithoutBInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "connectOrCreate",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "ACreateOrConnectWithoutBInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "connect",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "AWhereUniqueInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "AUncheckedCreateNestedOneWithoutBInput",
                      "meta": {
                        "grouping": "A"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "create",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "ACreateWithoutBInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "AUncheckedCreateWithoutBInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "connectOrCreate",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "ACreateOrConnectWithoutBInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "connect",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "AWhereUniqueInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "AUpdateOneWithoutBNestedInput",
                      "meta": {
                        "grouping": "A"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "create",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "ACreateWithoutBInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "AUncheckedCreateWithoutBInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "connectOrCreate",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "ACreateOrConnectWithoutBInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "upsert",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "AUpsertWithoutBInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "disconnect",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "Boolean",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "AWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "delete",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "Boolean",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "AWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "connect",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "AWhereUniqueInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "update",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "AUpdateToOneWithWhereWithoutBInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "AUpdateWithoutBInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "AUncheckedUpdateWithoutBInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "AUncheckedUpdateOneWithoutBNestedInput",
                      "meta": {
                        "grouping": "A"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "create",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "ACreateWithoutBInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "AUncheckedCreateWithoutBInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "connectOrCreate",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "ACreateOrConnectWithoutBInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "upsert",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "AUpsertWithoutBInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "disconnect",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "Boolean",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "AWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "delete",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "Boolean",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "AWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "connect",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "AWhereUniqueInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "update",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "AUpdateToOneWithWhereWithoutBInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "AUpdateWithoutBInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "AUncheckedUpdateWithoutBInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "NestedStringFilter",
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "equals",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "in",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": true
                            },
                            {
                              "type": "ListStringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "notIn",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": true
                            },
                            {
                              "type": "ListStringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "lt",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "lte",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "gt",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "gte",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "contains",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "startsWith",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "endsWith",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "not",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "NestedStringFilter",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "NestedStringWithAggregatesFilter",
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "equals",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "in",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": true
                            },
                            {
                              "type": "ListStringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "notIn",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": true
                            },
                            {
                              "type": "ListStringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "lt",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "lte",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "gt",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "gte",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "contains",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "startsWith",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "endsWith",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "not",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "NestedStringWithAggregatesFilter",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "_count",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "NestedIntFilter",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "_min",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "NestedStringFilter",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "_max",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "NestedStringFilter",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "NestedIntFilter",
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "equals",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "Int",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "IntFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "in",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "Int",
                              "location": "scalar",
                              "isList": true
                            },
                            {
                              "type": "ListIntFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "notIn",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "Int",
                              "location": "scalar",
                              "isList": true
                            },
                            {
                              "type": "ListIntFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "lt",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "Int",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "IntFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "lte",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "Int",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "IntFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "gt",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "Int",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "IntFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "gte",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "Int",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "IntFieldRefInput",
                              "namespace": "prisma",
                              "location": "fieldRefTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "not",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "Int",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "NestedIntFilter",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "BCreateWithoutAInput",
                      "meta": {
                        "grouping": "B"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": true,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "BUncheckedCreateWithoutAInput",
                      "meta": {
                        "grouping": "B"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": true,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "BCreateOrConnectWithoutAInput",
                      "meta": {
                        "grouping": "B"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "where",
                          "isRequired": true,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BWhereUniqueInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "create",
                          "isRequired": true,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BCreateWithoutAInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "BUncheckedCreateWithoutAInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "BUpsertWithoutAInput",
                      "meta": {
                        "grouping": "B"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "update",
                          "isRequired": true,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BUpdateWithoutAInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "BUncheckedUpdateWithoutAInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "create",
                          "isRequired": true,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BCreateWithoutAInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "BUncheckedCreateWithoutAInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "where",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "BUpdateToOneWithWhereWithoutAInput",
                      "meta": {
                        "grouping": "B"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "where",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "data",
                          "isRequired": true,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "BUpdateWithoutAInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "BUncheckedUpdateWithoutAInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "BUpdateWithoutAInput",
                      "meta": {
                        "grouping": "B"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldUpdateOperationsInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "BUncheckedUpdateWithoutAInput",
                      "meta": {
                        "grouping": "B"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldUpdateOperationsInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "ACreateWithoutBInput",
                      "meta": {
                        "grouping": "A"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": true,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "AUncheckedCreateWithoutBInput",
                      "meta": {
                        "grouping": "A"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": true,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "ACreateOrConnectWithoutBInput",
                      "meta": {
                        "grouping": "A"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "where",
                          "isRequired": true,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "AWhereUniqueInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "create",
                          "isRequired": true,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "ACreateWithoutBInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "AUncheckedCreateWithoutBInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "AUpsertWithoutBInput",
                      "meta": {
                        "grouping": "A"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "update",
                          "isRequired": true,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "AUpdateWithoutBInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "AUncheckedUpdateWithoutBInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "create",
                          "isRequired": true,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "ACreateWithoutBInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "AUncheckedCreateWithoutBInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "where",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "AWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "AUpdateToOneWithWhereWithoutBInput",
                      "meta": {
                        "grouping": "A"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "where",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "AWhereInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "data",
                          "isRequired": true,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "AUpdateWithoutBInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            },
                            {
                              "type": "AUncheckedUpdateWithoutBInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "AUpdateWithoutBInput",
                      "meta": {
                        "grouping": "A"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldUpdateOperationsInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "AUncheckedUpdateWithoutBInput",
                      "meta": {
                        "grouping": "A"
                      },
                      "constraints": {
                        "maxNumFields": null,
                        "minNumFields": null
                      },
                      "fields": [
                        {
                          "name": "id",
                          "isRequired": false,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            },
                            {
                              "type": "StringFieldUpdateOperationsInput",
                              "namespace": "prisma",
                              "location": "inputObjectTypes",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    }
                  ]
                },
                "outputObjectTypes": {
                  "prisma": [
                    {
                      "name": "Query",
                      "fields": [
                        {
                          "name": "findFirstA",
                          "args": [
                            {
                              "name": "where",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "AWhereInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "orderBy",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "AOrderByWithRelationInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": true
                                },
                                {
                                  "type": "AOrderByWithRelationInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "cursor",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "AWhereUniqueInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "take",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Int",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "skip",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Int",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "distinct",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "AScalarFieldEnum",
                                  "namespace": "prisma",
                                  "location": "enumTypes",
                                  "isList": false
                                },
                                {
                                  "type": "AScalarFieldEnum",
                                  "namespace": "prisma",
                                  "location": "enumTypes",
                                  "isList": true
                                }
                              ]
                            }
                          ],
                          "isNullable": true,
                          "outputType": {
                            "type": "A",
                            "namespace": "model",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        },
                        {
                          "name": "findFirstAOrThrow",
                          "args": [
                            {
                              "name": "where",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "AWhereInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "orderBy",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "AOrderByWithRelationInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": true
                                },
                                {
                                  "type": "AOrderByWithRelationInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "cursor",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "AWhereUniqueInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "take",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Int",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "skip",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Int",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "distinct",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "AScalarFieldEnum",
                                  "namespace": "prisma",
                                  "location": "enumTypes",
                                  "isList": false
                                },
                                {
                                  "type": "AScalarFieldEnum",
                                  "namespace": "prisma",
                                  "location": "enumTypes",
                                  "isList": true
                                }
                              ]
                            }
                          ],
                          "isNullable": true,
                          "outputType": {
                            "type": "A",
                            "namespace": "model",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        },
                        {
                          "name": "findManyA",
                          "args": [
                            {
                              "name": "where",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "AWhereInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "orderBy",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "AOrderByWithRelationInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": true
                                },
                                {
                                  "type": "AOrderByWithRelationInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "cursor",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "AWhereUniqueInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "take",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Int",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "skip",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Int",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "distinct",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "AScalarFieldEnum",
                                  "namespace": "prisma",
                                  "location": "enumTypes",
                                  "isList": false
                                },
                                {
                                  "type": "AScalarFieldEnum",
                                  "namespace": "prisma",
                                  "location": "enumTypes",
                                  "isList": true
                                }
                              ]
                            }
                          ],
                          "isNullable": false,
                          "outputType": {
                            "type": "A",
                            "namespace": "model",
                            "location": "outputObjectTypes",
                            "isList": true
                          }
                        },
                        {
                          "name": "aggregateA",
                          "args": [
                            {
                              "name": "where",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "AWhereInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "orderBy",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "AOrderByWithRelationInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": true
                                },
                                {
                                  "type": "AOrderByWithRelationInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "cursor",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "AWhereUniqueInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "take",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Int",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "skip",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Int",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            }
                          ],
                          "isNullable": false,
                          "outputType": {
                            "type": "AggregateA",
                            "namespace": "prisma",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        },
                        {
                          "name": "groupByA",
                          "args": [
                            {
                              "name": "where",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "AWhereInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "orderBy",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "AOrderByWithAggregationInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": true
                                },
                                {
                                  "type": "AOrderByWithAggregationInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "by",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "AScalarFieldEnum",
                                  "namespace": "prisma",
                                  "location": "enumTypes",
                                  "isList": true
                                },
                                {
                                  "type": "AScalarFieldEnum",
                                  "namespace": "prisma",
                                  "location": "enumTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "having",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "AScalarWhereWithAggregatesInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "take",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Int",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "skip",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Int",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            }
                          ],
                          "isNullable": false,
                          "outputType": {
                            "type": "AGroupByOutputType",
                            "namespace": "prisma",
                            "location": "outputObjectTypes",
                            "isList": true
                          }
                        },
                        {
                          "name": "findUniqueA",
                          "args": [
                            {
                              "name": "where",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "AWhereUniqueInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            }
                          ],
                          "isNullable": true,
                          "outputType": {
                            "type": "A",
                            "namespace": "model",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        },
                        {
                          "name": "findUniqueAOrThrow",
                          "args": [
                            {
                              "name": "where",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "AWhereUniqueInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            }
                          ],
                          "isNullable": true,
                          "outputType": {
                            "type": "A",
                            "namespace": "model",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        },
                        {
                          "name": "findFirstB",
                          "args": [
                            {
                              "name": "where",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BWhereInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "orderBy",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BOrderByWithRelationInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": true
                                },
                                {
                                  "type": "BOrderByWithRelationInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "cursor",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BWhereUniqueInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "take",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Int",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "skip",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Int",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "distinct",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BScalarFieldEnum",
                                  "namespace": "prisma",
                                  "location": "enumTypes",
                                  "isList": false
                                },
                                {
                                  "type": "BScalarFieldEnum",
                                  "namespace": "prisma",
                                  "location": "enumTypes",
                                  "isList": true
                                }
                              ]
                            }
                          ],
                          "isNullable": true,
                          "outputType": {
                            "type": "B",
                            "namespace": "model",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        },
                        {
                          "name": "findFirstBOrThrow",
                          "args": [
                            {
                              "name": "where",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BWhereInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "orderBy",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BOrderByWithRelationInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": true
                                },
                                {
                                  "type": "BOrderByWithRelationInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "cursor",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BWhereUniqueInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "take",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Int",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "skip",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Int",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "distinct",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BScalarFieldEnum",
                                  "namespace": "prisma",
                                  "location": "enumTypes",
                                  "isList": false
                                },
                                {
                                  "type": "BScalarFieldEnum",
                                  "namespace": "prisma",
                                  "location": "enumTypes",
                                  "isList": true
                                }
                              ]
                            }
                          ],
                          "isNullable": true,
                          "outputType": {
                            "type": "B",
                            "namespace": "model",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        },
                        {
                          "name": "findManyB",
                          "args": [
                            {
                              "name": "where",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BWhereInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "orderBy",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BOrderByWithRelationInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": true
                                },
                                {
                                  "type": "BOrderByWithRelationInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "cursor",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BWhereUniqueInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "take",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Int",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "skip",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Int",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "distinct",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BScalarFieldEnum",
                                  "namespace": "prisma",
                                  "location": "enumTypes",
                                  "isList": false
                                },
                                {
                                  "type": "BScalarFieldEnum",
                                  "namespace": "prisma",
                                  "location": "enumTypes",
                                  "isList": true
                                }
                              ]
                            }
                          ],
                          "isNullable": false,
                          "outputType": {
                            "type": "B",
                            "namespace": "model",
                            "location": "outputObjectTypes",
                            "isList": true
                          }
                        },
                        {
                          "name": "aggregateB",
                          "args": [
                            {
                              "name": "where",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BWhereInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "orderBy",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BOrderByWithRelationInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": true
                                },
                                {
                                  "type": "BOrderByWithRelationInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "cursor",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BWhereUniqueInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "take",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Int",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "skip",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Int",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            }
                          ],
                          "isNullable": false,
                          "outputType": {
                            "type": "AggregateB",
                            "namespace": "prisma",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        },
                        {
                          "name": "groupByB",
                          "args": [
                            {
                              "name": "where",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BWhereInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "orderBy",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BOrderByWithAggregationInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": true
                                },
                                {
                                  "type": "BOrderByWithAggregationInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "by",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BScalarFieldEnum",
                                  "namespace": "prisma",
                                  "location": "enumTypes",
                                  "isList": true
                                },
                                {
                                  "type": "BScalarFieldEnum",
                                  "namespace": "prisma",
                                  "location": "enumTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "having",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BScalarWhereWithAggregatesInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "take",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Int",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "skip",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Int",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            }
                          ],
                          "isNullable": false,
                          "outputType": {
                            "type": "BGroupByOutputType",
                            "namespace": "prisma",
                            "location": "outputObjectTypes",
                            "isList": true
                          }
                        },
                        {
                          "name": "findUniqueB",
                          "args": [
                            {
                              "name": "where",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BWhereUniqueInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            }
                          ],
                          "isNullable": true,
                          "outputType": {
                            "type": "B",
                            "namespace": "model",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        },
                        {
                          "name": "findUniqueBOrThrow",
                          "args": [
                            {
                              "name": "where",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BWhereUniqueInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            }
                          ],
                          "isNullable": true,
                          "outputType": {
                            "type": "B",
                            "namespace": "model",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        }
                      ]
                    },
                    {
                      "name": "Mutation",
                      "fields": [
                        {
                          "name": "createOneA",
                          "args": [
                            {
                              "name": "data",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "ACreateInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                },
                                {
                                  "type": "AUncheckedCreateInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            }
                          ],
                          "isNullable": false,
                          "outputType": {
                            "type": "A",
                            "namespace": "model",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        },
                        {
                          "name": "upsertOneA",
                          "args": [
                            {
                              "name": "where",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "AWhereUniqueInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "create",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "ACreateInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                },
                                {
                                  "type": "AUncheckedCreateInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "update",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "AUpdateInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                },
                                {
                                  "type": "AUncheckedUpdateInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            }
                          ],
                          "isNullable": false,
                          "outputType": {
                            "type": "A",
                            "namespace": "model",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        },
                        {
                          "name": "createManyA",
                          "args": [
                            {
                              "name": "data",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "ACreateManyInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                },
                                {
                                  "type": "ACreateManyInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": true
                                }
                              ]
                            },
                            {
                              "name": "skipDuplicates",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Boolean",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            }
                          ],
                          "isNullable": false,
                          "outputType": {
                            "type": "AffectedRowsOutput",
                            "namespace": "prisma",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        },
                        {
                          "name": "createManyAAndReturn",
                          "args": [
                            {
                              "name": "data",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "ACreateManyInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                },
                                {
                                  "type": "ACreateManyInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": true
                                }
                              ]
                            },
                            {
                              "name": "skipDuplicates",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Boolean",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            }
                          ],
                          "isNullable": false,
                          "outputType": {
                            "type": "CreateManyAAndReturnOutputType",
                            "namespace": "model",
                            "location": "outputObjectTypes",
                            "isList": true
                          }
                        },
                        {
                          "name": "deleteOneA",
                          "args": [
                            {
                              "name": "where",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "AWhereUniqueInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            }
                          ],
                          "isNullable": true,
                          "outputType": {
                            "type": "A",
                            "namespace": "model",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        },
                        {
                          "name": "updateOneA",
                          "args": [
                            {
                              "name": "data",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "AUpdateInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                },
                                {
                                  "type": "AUncheckedUpdateInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "where",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "AWhereUniqueInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            }
                          ],
                          "isNullable": true,
                          "outputType": {
                            "type": "A",
                            "namespace": "model",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        },
                        {
                          "name": "updateManyA",
                          "args": [
                            {
                              "name": "data",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "AUpdateManyMutationInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                },
                                {
                                  "type": "AUncheckedUpdateManyInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "where",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "AWhereInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "limit",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Int",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            }
                          ],
                          "isNullable": false,
                          "outputType": {
                            "type": "AffectedRowsOutput",
                            "namespace": "prisma",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        },
                        {
                          "name": "updateManyAAndReturn",
                          "args": [
                            {
                              "name": "data",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "AUpdateManyMutationInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                },
                                {
                                  "type": "AUncheckedUpdateManyInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "where",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "AWhereInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "limit",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Int",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            }
                          ],
                          "isNullable": false,
                          "outputType": {
                            "type": "UpdateManyAAndReturnOutputType",
                            "namespace": "model",
                            "location": "outputObjectTypes",
                            "isList": true
                          }
                        },
                        {
                          "name": "deleteManyA",
                          "args": [
                            {
                              "name": "where",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "AWhereInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "limit",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Int",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            }
                          ],
                          "isNullable": false,
                          "outputType": {
                            "type": "AffectedRowsOutput",
                            "namespace": "prisma",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        },
                        {
                          "name": "createOneB",
                          "args": [
                            {
                              "name": "data",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BCreateInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                },
                                {
                                  "type": "BUncheckedCreateInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            }
                          ],
                          "isNullable": false,
                          "outputType": {
                            "type": "B",
                            "namespace": "model",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        },
                        {
                          "name": "upsertOneB",
                          "args": [
                            {
                              "name": "where",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BWhereUniqueInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "create",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BCreateInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                },
                                {
                                  "type": "BUncheckedCreateInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "update",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BUpdateInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                },
                                {
                                  "type": "BUncheckedUpdateInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            }
                          ],
                          "isNullable": false,
                          "outputType": {
                            "type": "B",
                            "namespace": "model",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        },
                        {
                          "name": "createManyB",
                          "args": [
                            {
                              "name": "data",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BCreateManyInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                },
                                {
                                  "type": "BCreateManyInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": true
                                }
                              ]
                            },
                            {
                              "name": "skipDuplicates",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Boolean",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            }
                          ],
                          "isNullable": false,
                          "outputType": {
                            "type": "AffectedRowsOutput",
                            "namespace": "prisma",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        },
                        {
                          "name": "createManyBAndReturn",
                          "args": [
                            {
                              "name": "data",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BCreateManyInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                },
                                {
                                  "type": "BCreateManyInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": true
                                }
                              ]
                            },
                            {
                              "name": "skipDuplicates",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Boolean",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            }
                          ],
                          "isNullable": false,
                          "outputType": {
                            "type": "CreateManyBAndReturnOutputType",
                            "namespace": "model",
                            "location": "outputObjectTypes",
                            "isList": true
                          }
                        },
                        {
                          "name": "deleteOneB",
                          "args": [
                            {
                              "name": "where",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BWhereUniqueInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            }
                          ],
                          "isNullable": true,
                          "outputType": {
                            "type": "B",
                            "namespace": "model",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        },
                        {
                          "name": "updateOneB",
                          "args": [
                            {
                              "name": "data",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BUpdateInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                },
                                {
                                  "type": "BUncheckedUpdateInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "where",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BWhereUniqueInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            }
                          ],
                          "isNullable": true,
                          "outputType": {
                            "type": "B",
                            "namespace": "model",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        },
                        {
                          "name": "updateManyB",
                          "args": [
                            {
                              "name": "data",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BUpdateManyMutationInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                },
                                {
                                  "type": "BUncheckedUpdateManyInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "where",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BWhereInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "limit",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Int",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            }
                          ],
                          "isNullable": false,
                          "outputType": {
                            "type": "AffectedRowsOutput",
                            "namespace": "prisma",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        },
                        {
                          "name": "updateManyBAndReturn",
                          "args": [
                            {
                              "name": "data",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BUpdateManyMutationInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                },
                                {
                                  "type": "BUncheckedUpdateManyInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "where",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BWhereInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "limit",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Int",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            }
                          ],
                          "isNullable": false,
                          "outputType": {
                            "type": "UpdateManyBAndReturnOutputType",
                            "namespace": "model",
                            "location": "outputObjectTypes",
                            "isList": true
                          }
                        },
                        {
                          "name": "deleteManyB",
                          "args": [
                            {
                              "name": "where",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "BWhereInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "limit",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Int",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            }
                          ],
                          "isNullable": false,
                          "outputType": {
                            "type": "AffectedRowsOutput",
                            "namespace": "prisma",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        },
                        {
                          "name": "executeRaw",
                          "args": [
                            {
                              "name": "query",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "String",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "parameters",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Json",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            }
                          ],
                          "isNullable": false,
                          "outputType": {
                            "type": "Json",
                            "location": "scalar",
                            "isList": false
                          }
                        },
                        {
                          "name": "queryRaw",
                          "args": [
                            {
                              "name": "query",
                              "isRequired": true,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "String",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            },
                            {
                              "name": "parameters",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "Json",
                                  "location": "scalar",
                                  "isList": false
                                }
                              ]
                            }
                          ],
                          "isNullable": false,
                          "outputType": {
                            "type": "Json",
                            "location": "scalar",
                            "isList": false
                          }
                        }
                      ]
                    },
                    {
                      "name": "AggregateA",
                      "fields": [
                        {
                          "name": "_count",
                          "args": [],
                          "isNullable": true,
                          "outputType": {
                            "type": "ACountAggregateOutputType",
                            "namespace": "prisma",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        },
                        {
                          "name": "_min",
                          "args": [],
                          "isNullable": true,
                          "outputType": {
                            "type": "AMinAggregateOutputType",
                            "namespace": "prisma",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        },
                        {
                          "name": "_max",
                          "args": [],
                          "isNullable": true,
                          "outputType": {
                            "type": "AMaxAggregateOutputType",
                            "namespace": "prisma",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        }
                      ]
                    },
                    {
                      "name": "AGroupByOutputType",
                      "fields": [
                        {
                          "name": "id",
                          "args": [],
                          "isNullable": false,
                          "outputType": {
                            "type": "String",
                            "location": "scalar",
                            "isList": false
                          }
                        },
                        {
                          "name": "b_id",
                          "args": [],
                          "isNullable": false,
                          "outputType": {
                            "type": "String",
                            "location": "scalar",
                            "isList": false
                          }
                        },
                        {
                          "name": "_count",
                          "args": [],
                          "isNullable": true,
                          "outputType": {
                            "type": "ACountAggregateOutputType",
                            "namespace": "prisma",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        },
                        {
                          "name": "_min",
                          "args": [],
                          "isNullable": true,
                          "outputType": {
                            "type": "AMinAggregateOutputType",
                            "namespace": "prisma",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        },
                        {
                          "name": "_max",
                          "args": [],
                          "isNullable": true,
                          "outputType": {
                            "type": "AMaxAggregateOutputType",
                            "namespace": "prisma",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        }
                      ]
                    },
                    {
                      "name": "AggregateB",
                      "fields": [
                        {
                          "name": "_count",
                          "args": [],
                          "isNullable": true,
                          "outputType": {
                            "type": "BCountAggregateOutputType",
                            "namespace": "prisma",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        },
                        {
                          "name": "_min",
                          "args": [],
                          "isNullable": true,
                          "outputType": {
                            "type": "BMinAggregateOutputType",
                            "namespace": "prisma",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        },
                        {
                          "name": "_max",
                          "args": [],
                          "isNullable": true,
                          "outputType": {
                            "type": "BMaxAggregateOutputType",
                            "namespace": "prisma",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        }
                      ]
                    },
                    {
                      "name": "BGroupByOutputType",
                      "fields": [
                        {
                          "name": "id",
                          "args": [],
                          "isNullable": false,
                          "outputType": {
                            "type": "String",
                            "location": "scalar",
                            "isList": false
                          }
                        },
                        {
                          "name": "_count",
                          "args": [],
                          "isNullable": true,
                          "outputType": {
                            "type": "BCountAggregateOutputType",
                            "namespace": "prisma",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        },
                        {
                          "name": "_min",
                          "args": [],
                          "isNullable": true,
                          "outputType": {
                            "type": "BMinAggregateOutputType",
                            "namespace": "prisma",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        },
                        {
                          "name": "_max",
                          "args": [],
                          "isNullable": true,
                          "outputType": {
                            "type": "BMaxAggregateOutputType",
                            "namespace": "prisma",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        }
                      ]
                    },
                    {
                      "name": "AffectedRowsOutput",
                      "fields": [
                        {
                          "name": "count",
                          "args": [],
                          "isNullable": false,
                          "outputType": {
                            "type": "Int",
                            "location": "scalar",
                            "isList": false
                          }
                        }
                      ]
                    },
                    {
                      "name": "ACountAggregateOutputType",
                      "fields": [
                        {
                          "name": "id",
                          "args": [],
                          "isNullable": false,
                          "outputType": {
                            "type": "Int",
                            "location": "scalar",
                            "isList": false
                          }
                        },
                        {
                          "name": "b_id",
                          "args": [],
                          "isNullable": false,
                          "outputType": {
                            "type": "Int",
                            "location": "scalar",
                            "isList": false
                          }
                        },
                        {
                          "name": "_all",
                          "args": [],
                          "isNullable": false,
                          "outputType": {
                            "type": "Int",
                            "location": "scalar",
                            "isList": false
                          }
                        }
                      ]
                    },
                    {
                      "name": "AMinAggregateOutputType",
                      "fields": [
                        {
                          "name": "id",
                          "args": [],
                          "isNullable": true,
                          "outputType": {
                            "type": "String",
                            "location": "scalar",
                            "isList": false
                          }
                        },
                        {
                          "name": "b_id",
                          "args": [],
                          "isNullable": true,
                          "outputType": {
                            "type": "String",
                            "location": "scalar",
                            "isList": false
                          }
                        }
                      ]
                    },
                    {
                      "name": "AMaxAggregateOutputType",
                      "fields": [
                        {
                          "name": "id",
                          "args": [],
                          "isNullable": true,
                          "outputType": {
                            "type": "String",
                            "location": "scalar",
                            "isList": false
                          }
                        },
                        {
                          "name": "b_id",
                          "args": [],
                          "isNullable": true,
                          "outputType": {
                            "type": "String",
                            "location": "scalar",
                            "isList": false
                          }
                        }
                      ]
                    },
                    {
                      "name": "BCountAggregateOutputType",
                      "fields": [
                        {
                          "name": "id",
                          "args": [],
                          "isNullable": false,
                          "outputType": {
                            "type": "Int",
                            "location": "scalar",
                            "isList": false
                          }
                        },
                        {
                          "name": "_all",
                          "args": [],
                          "isNullable": false,
                          "outputType": {
                            "type": "Int",
                            "location": "scalar",
                            "isList": false
                          }
                        }
                      ]
                    },
                    {
                      "name": "BMinAggregateOutputType",
                      "fields": [
                        {
                          "name": "id",
                          "args": [],
                          "isNullable": true,
                          "outputType": {
                            "type": "String",
                            "location": "scalar",
                            "isList": false
                          }
                        }
                      ]
                    },
                    {
                      "name": "BMaxAggregateOutputType",
                      "fields": [
                        {
                          "name": "id",
                          "args": [],
                          "isNullable": true,
                          "outputType": {
                            "type": "String",
                            "location": "scalar",
                            "isList": false
                          }
                        }
                      ]
                    }
                  ],
                  "model": [
                    {
                      "name": "A",
                      "fields": [
                        {
                          "name": "id",
                          "args": [],
                          "isNullable": false,
                          "outputType": {
                            "type": "String",
                            "location": "scalar",
                            "isList": false
                          }
                        },
                        {
                          "name": "b_id",
                          "args": [],
                          "isNullable": false,
                          "outputType": {
                            "type": "String",
                            "location": "scalar",
                            "isList": false
                          }
                        },
                        {
                          "name": "b",
                          "args": [],
                          "isNullable": false,
                          "outputType": {
                            "type": "B",
                            "namespace": "model",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        }
                      ]
                    },
                    {
                      "name": "B",
                      "fields": [
                        {
                          "name": "id",
                          "args": [],
                          "isNullable": false,
                          "outputType": {
                            "type": "String",
                            "location": "scalar",
                            "isList": false
                          }
                        },
                        {
                          "name": "a",
                          "args": [
                            {
                              "name": "where",
                              "isRequired": false,
                              "isNullable": false,
                              "inputTypes": [
                                {
                                  "type": "AWhereInput",
                                  "namespace": "prisma",
                                  "location": "inputObjectTypes",
                                  "isList": false
                                }
                              ]
                            }
                          ],
                          "isNullable": true,
                          "outputType": {
                            "type": "A",
                            "namespace": "model",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        }
                      ]
                    },
                    {
                      "name": "CreateManyAAndReturnOutputType",
                      "fields": [
                        {
                          "name": "id",
                          "args": [],
                          "isNullable": false,
                          "outputType": {
                            "type": "String",
                            "location": "scalar",
                            "isList": false
                          }
                        },
                        {
                          "name": "b_id",
                          "args": [],
                          "isNullable": false,
                          "outputType": {
                            "type": "String",
                            "location": "scalar",
                            "isList": false
                          }
                        },
                        {
                          "name": "b",
                          "args": [],
                          "isNullable": false,
                          "outputType": {
                            "type": "B",
                            "namespace": "model",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        }
                      ]
                    },
                    {
                      "name": "UpdateManyAAndReturnOutputType",
                      "fields": [
                        {
                          "name": "id",
                          "args": [],
                          "isNullable": false,
                          "outputType": {
                            "type": "String",
                            "location": "scalar",
                            "isList": false
                          }
                        },
                        {
                          "name": "b_id",
                          "args": [],
                          "isNullable": false,
                          "outputType": {
                            "type": "String",
                            "location": "scalar",
                            "isList": false
                          }
                        },
                        {
                          "name": "b",
                          "args": [],
                          "isNullable": false,
                          "outputType": {
                            "type": "B",
                            "namespace": "model",
                            "location": "outputObjectTypes",
                            "isList": false
                          }
                        }
                      ]
                    },
                    {
                      "name": "CreateManyBAndReturnOutputType",
                      "fields": [
                        {
                          "name": "id",
                          "args": [],
                          "isNullable": false,
                          "outputType": {
                            "type": "String",
                            "location": "scalar",
                            "isList": false
                          }
                        }
                      ]
                    },
                    {
                      "name": "UpdateManyBAndReturnOutputType",
                      "fields": [
                        {
                          "name": "id",
                          "args": [],
                          "isNullable": false,
                          "outputType": {
                            "type": "String",
                            "location": "scalar",
                            "isList": false
                          }
                        }
                      ]
                    }
                  ]
                },
                "enumTypes": {
                  "prisma": [
                    {
                      "name": "TransactionIsolationLevel",
                      "values": [
                        "ReadUncommitted",
                        "ReadCommitted",
                        "RepeatableRead",
                        "Serializable"
                      ]
                    },
                    {
                      "name": "AScalarFieldEnum",
                      "values": [
                        "id",
                        "b_id"
                      ]
                    },
                    {
                      "name": "BScalarFieldEnum",
                      "values": [
                        "id"
                      ]
                    },
                    {
                      "name": "SortOrder",
                      "values": [
                        "asc",
                        "desc"
                      ]
                    },
                    {
                      "name": "QueryMode",
                      "values": [
                        "default",
                        "insensitive"
                      ]
                    }
                  ]
                },
                "fieldRefTypes": {
                  "prisma": [
                    {
                      "name": "StringFieldRefInput",
                      "allowTypes": [
                        {
                          "type": "String",
                          "location": "scalar",
                          "isList": false
                        }
                      ],
                      "fields": [
                        {
                          "name": "_ref",
                          "isRequired": true,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "_container",
                          "isRequired": true,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "ListStringFieldRefInput",
                      "allowTypes": [
                        {
                          "type": "String",
                          "location": "scalar",
                          "isList": true
                        }
                      ],
                      "fields": [
                        {
                          "name": "_ref",
                          "isRequired": true,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "_container",
                          "isRequired": true,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "IntFieldRefInput",
                      "allowTypes": [
                        {
                          "type": "Int",
                          "location": "scalar",
                          "isList": false
                        }
                      ],
                      "fields": [
                        {
                          "name": "_ref",
                          "isRequired": true,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "_container",
                          "isRequired": true,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "ListIntFieldRefInput",
                      "allowTypes": [
                        {
                          "type": "Int",
                          "location": "scalar",
                          "isList": true
                        }
                      ],
                      "fields": [
                        {
                          "name": "_ref",
                          "isRequired": true,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        },
                        {
                          "name": "_container",
                          "isRequired": true,
                          "isNullable": false,
                          "inputTypes": [
                            {
                              "type": "String",
                              "location": "scalar",
                              "isList": false
                            }
                          ]
                        }
                      ]
                    }
                  ]
                }
              },
              "mappings": {
                "modelOperations": [
                  {
                    "model": "A",
                    "aggregate": "aggregateA",
                    "createMany": "createManyA",
                    "createManyAndReturn": "createManyAAndReturn",
                    "createOne": "createOneA",
                    "deleteMany": "deleteManyA",
                    "deleteOne": "deleteOneA",
                    "findFirst": "findFirstA",
                    "findFirstOrThrow": "findFirstAOrThrow",
                    "findMany": "findManyA",
                    "findUnique": "findUniqueA",
                    "findUniqueOrThrow": "findUniqueAOrThrow",
                    "groupBy": "groupByA",
                    "updateMany": "updateManyA",
                    "updateManyAndReturn": "updateManyAAndReturn",
                    "updateOne": "updateOneA",
                    "upsertOne": "upsertOneA"
                  },
                  {
                    "model": "B",
                    "aggregate": "aggregateB",
                    "createMany": "createManyB",
                    "createManyAndReturn": "createManyBAndReturn",
                    "createOne": "createOneB",
                    "deleteMany": "deleteManyB",
                    "deleteOne": "deleteOneB",
                    "findFirst": "findFirstB",
                    "findFirstOrThrow": "findFirstBOrThrow",
                    "findMany": "findManyB",
                    "findUnique": "findUniqueB",
                    "findUniqueOrThrow": "findUniqueBOrThrow",
                    "groupBy": "groupByB",
                    "updateMany": "updateManyB",
                    "updateManyAndReturn": "updateManyBAndReturn",
                    "updateOne": "updateOneB",
                    "upsertOne": "upsertOneB"
                  }
                ],
                "otherOperations": {
                  "read": [],
                  "write": [
                    "executeRaw",
                    "queryRaw"
                  ]
                }
              }
            }"#]];

        let response = get_dmmf(&request.to_string()).unwrap();

        let prettified_response =
            serde_json::to_string_pretty(&serde_json::from_str::<serde_json::Value>(&response).unwrap()).unwrap();

        expected.assert_eq(&prettified_response);
    }

    #[test]
    fn get_dmmf_using_both_relation_mode_and_referential_integrity() {
        let schema = r#"
          datasource db {
              provider = "sqlite"
              url = "sqlite"
              relationMode = "prisma"
              referentialIntegrity = "foreignKeys"
          }
        "#;

        let request = json!({
            "prismaSchema": schema,
        });

        let expected = expect![[
            r#"{"error_code":"P1012","message":"\u001b[1;91merror\u001b[0m: \u001b[1mThe `referentialIntegrity` and `relationMode` attributes cannot be used together. Please use only `relationMode` instead.\u001b[0m\n  \u001b[1;94m-->\u001b[0m  \u001b[4mschema.prisma:6\u001b[0m\n\u001b[1;94m   | \u001b[0m\n\u001b[1;94m 5 | \u001b[0m              relationMode = \"prisma\"\n\u001b[1;94m 6 | \u001b[0m              \u001b[1;91mreferentialIntegrity = \"foreignKeys\"\u001b[0m\n\u001b[1;94m   | \u001b[0m\n\nValidation Error Count: 1"}"#
        ]];
        let response = get_dmmf(&request.to_string()).unwrap_err();
        expected.assert_eq(&response);
    }
}
