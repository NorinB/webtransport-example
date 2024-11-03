/*
 * Public API Surface of wasm-webtransport
 */

import init from './pkg';
export {
  WebTransportClient,
  WebTransportBistream,
  WebTransportSendStream,
  WebTransportReceiveStream,
} from './pkg';
export { init as initWasm };
