import * as web3 from '@solana/web3.js'
export * from './consts'

export function proxyConnection() {
  return new web3.Connection('http://127.0.0.1:9899', 'confirmed')
}
