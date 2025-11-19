use crate::proto::{
    CandleRequest, FillRequest, InitRequest, InitResponse, OrderBookRequest, SignalList,
    TickRequest,
};
use anyhow::Result;

/// Transport-agnostic interface for communicating with external strategies.
///
/// This allows swapping gRPC for Shared Memory, ZeroMQ, or other transports
/// without changing the core RpcStrategy logic.
pub trait RemoteStrategyClient: Send + Sync {
    /// Establishes the connection to the remote strategy.
    fn connect(&mut self) -> Result<()>;

    /// Performs the initial handshake and configuration.
    fn initialize(&mut self, req: InitRequest) -> Result<InitResponse>;

    /// Pushes a tick event.
    fn on_tick(&mut self, req: TickRequest) -> Result<SignalList>;

    /// Pushes a candle event.
    fn on_candle(&mut self, req: CandleRequest) -> Result<SignalList>;

    /// Pushes an order book snapshot.
    fn on_order_book(&mut self, req: OrderBookRequest) -> Result<SignalList>;

    /// Pushes an execution fill.
    fn on_fill(&mut self, req: FillRequest) -> Result<SignalList>;
}
