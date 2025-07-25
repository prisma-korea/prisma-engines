import * as readline from 'node:readline'
import * as S from '@effect/schema/Schema'
import {
  bindAdapter,
  ErrorCapturingSqlDriverAdapter,
} from '@prisma/driver-adapter-utils'

import type { DriverAdaptersManager } from './driver-adapters-manager/index.js'
import { Env, jsonRpc } from './types/index.js'
import * as qe from './query-engine.js'
import { nextRequestId } from './requestId.js'
import { createRNEngineConnector } from './rn.js'
import { assertNever, debug, err } from './utils.js'
import { setupDriverAdaptersManager } from './setup.js'
import { SchemaId } from './types/jsonRpc.js'
import { setupDefaultPanicHandler } from './panic.js'

async function main(): Promise<void> {
  const env = S.decodeUnknownSync(Env)(process.env)
  console.log('[env]', env)

  setupDefaultPanicHandler()

  const iface = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
    terminal: false,
  })

  iface.on('line', async (line) => {
    try {
      const request = S.decodeSync(jsonRpc.RequestFromString)(line)
      debug(`Got a request: ${line}`)

      try {
        const response = await handleRequest(request, env)
        respondOk(request.id, response)
      } catch (err) {
        debug('[nodejs] Error from request handler: ', err)
        respondErr(request.id, {
          code: 1,
          message: err.stack ?? err.toString(),
        })
      }
    } catch (err) {
      debug('Received non-json line: ', line)
      console.error(err)
    }
  })
}

const state: Record<
  SchemaId,
  {
    engine: qe.QueryEngine
    driverAdapterManager: DriverAdaptersManager
    adapter: ErrorCapturingSqlDriverAdapter | null
    logs: string[]
  }
> = {}

async function handleRequest(
  { method, params }: jsonRpc.Request,
  env: Env,
): Promise<unknown> {
  if (method !== 'initializeSchema') {
    if (state[params.schemaId] === undefined) {
      throw new Error(
        `Schema with id ${params.schemaId} is not initialized. Please call 'initializeSchema' first.`,
      )
    }
  }

  switch (method) {
    case 'initializeSchema': {
      debug('Got `initializeSchema', params)

      const { url, schema, schemaId, migrationScript } = params
      const logs = [] as string[]

      const logCallback = (log) => {
        logs.push(log)
      }

      const driverAdapterManager = await setupDriverAdaptersManager(env, {
        url,
        migrationScript,
      })

      const { engine, adapter } = await initQe({
        env,
        url,
        driverAdapterManager,
        schema,
        logCallback,
      })
      await engine.connect('', nextRequestId())

      state[schemaId] = {
        engine,
        driverAdapterManager,
        adapter,
        logs,
      }

      if (adapter && adapter.getConnectionInfo) {
        const maxBindValuesResult = adapter
          .getConnectionInfo()
          .map((info) => info.maxBindValues)
        if (maxBindValuesResult.ok) {
          return { maxBindValues: maxBindValuesResult.value }
        }
      }

      return { maxBindValues: null }
    }
    case 'query': {
      debug('Got `query`', params)
      const { query, schemaId, txId } = params
      const engine = state[schemaId].engine
      const result = await engine.query(
        JSON.stringify(query),
        '',
        txId ?? undefined,
        nextRequestId(),
      )

      const parsedResult = JSON.parse(result)
      if (parsedResult.errors) {
        const error = parsedResult.errors[0]?.user_facing_error
        if (error.error_code === 'P2036') {
          const jsError = state[schemaId].adapter?.errorRegistry.consumeError(
            error.meta.id,
          )
          if (!jsError) {
            err(
              `Something went wrong. Engine reported external error with id ${error.meta.id}, but it was not registered.`,
            )
          } else {
            err(
              'got error response from the engine caused by the driver: ',
              jsError,
            )
          }
        }
      }

      debug('🟢 Engine response: ', result)
      // returning unparsed string: otherwise, some information gots lost during this round-trip.
      // In particular, floating point without decimal part turn into integers
      return result
    }

    case 'startTx': {
      debug('Got `startTx', params)
      const { schemaId, options } = params
      const result = await state[schemaId].engine.startTransaction(
        JSON.stringify(options),
        '',
        nextRequestId(),
      )
      return JSON.parse(result)
    }

    case 'commitTx': {
      debug('Got `commitTx', params)
      const { schemaId, txId } = params
      const result = await state[schemaId].engine.commitTransaction(
        txId,
        '{}',
        nextRequestId(),
      )
      return JSON.parse(result)
    }

    case 'rollbackTx': {
      debug('Got `rollbackTx', params)
      const { schemaId, txId } = params
      const result = await state[schemaId].engine.rollbackTransaction(
        txId,
        '{}',
        nextRequestId(),
      )
      return JSON.parse(result)
    }
    case 'teardown': {
      debug('Got `teardown', params)
      const { schemaId } = params
      const { engine, driverAdapterManager } = state[schemaId]

      await engine.disconnect('', nextRequestId())

      await driverAdapterManager.teardown()

      delete state[schemaId]

      return {}
    }
    case 'getLogs': {
      const { schemaId } = params
      const clonedLogs = [...state[schemaId].logs]
      state[schemaId].logs.length = 0
      return clonedLogs
    }
    default: {
      assertNever(method, `Unknown method: \`${method}\``)
    }
  }
}

function respondErr(requestId: number, error: jsonRpc.RpcError) {
  const msg: jsonRpc.ErrResponse = {
    jsonrpc: '2.0',
    id: requestId,
    error,
  }
  console.log(JSON.stringify(msg))
}

function respondOk(requestId: number, payload: unknown) {
  const msg: jsonRpc.OkResponse = {
    jsonrpc: '2.0',
    id: requestId,
    result: payload,
  }
  console.log(JSON.stringify(msg))
}

type InitQueryEngineParams = {
  env: Env
  driverAdapterManager: DriverAdaptersManager
  url: string
  schema: string
  logCallback: qe.QueryLogCallback
}

async function initQe({
  env,
  driverAdapterManager,
  url,
  schema,
  logCallback,
}: InitQueryEngineParams) {
  if (env.EXTERNAL_TEST_EXECUTOR === 'Mobile') {
    url = env.MOBILE_EMULATOR_URL

    const engine = createRNEngineConnector(url, schema, logCallback)
    return { engine, adapter: null }
  }

  if (env.EXTERNAL_TEST_EXECUTOR === 'QueryCompiler') {
    throw new Error('query compiler tests must be run using testd-qc.ts')
  }

  const adapter = await driverAdapterManager.connect()
  const errorCapturingAdapter = bindAdapter(adapter)
  const engineInstance = await qe.initQueryEngine(
    env.EXTERNAL_TEST_EXECUTOR,
    errorCapturingAdapter,
    schema,
    driverAdapterManager.connector(),
    logCallback,
    debug,
  )

  return {
    engine: engineInstance,
    adapter: errorCapturingAdapter,
  }
}

main().catch(err)
