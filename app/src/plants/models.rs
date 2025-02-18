use std::{collections::HashMap, future::Future};

use tokio::sync::{mpsc, oneshot};

use crate::game::delivery_period::DeliveryPeriodId;

use super::{
    stack::{ProgramPlant, StackMessage},
    PlantId, PlantOutput, PowerPlantPublicRepr,
};

#[derive(Debug)]
pub struct CloseStackError(DeliveryPeriodId);
#[derive(Debug)]
pub struct GetSnashotError;

pub trait StackService: Clone + Send + Sync + 'static {
    /// Open the stack so that its plants can be programmed.
    fn open_stack(&self, delivery_period: DeliveryPeriodId) -> impl Future<Output = ()> + Send;

    /// Close the stack and disptach its plants based on their last setpoints
    fn close_stack(
        &self,
        delivery_period: DeliveryPeriodId,
    ) -> impl Future<Output = Result<HashMap<PlantId, PlantOutput>, CloseStackError>> + Send;

    /// Program a setpoint on a power plant of the stack
    fn program_setpoint(&self, plant: PlantId, setpoint: isize) -> impl Future<Output = ()> + Send;

    /// Get a snapshot of the stack's power plant current setpoint and cost
    fn get_snapshot(
        &self,
    ) -> impl Future<Output = Result<HashMap<PlantId, PowerPlantPublicRepr>, GetSnashotError>> + Send;
}

#[derive(Debug, Clone)]
pub struct Service {
    tx: mpsc::Sender<StackMessage>,
}

impl Service {
    pub fn new(tx: mpsc::Sender<StackMessage>) -> Service {
        Service { tx }
    }
}

impl StackService for Service {
    async fn open_stack(&self, delivery_period: DeliveryPeriodId) {
        let _ = self.tx.send(StackMessage::OpenStack(delivery_period)).await;
    }

    async fn close_stack(
        &self,
        delivery_period: DeliveryPeriodId,
    ) -> Result<HashMap<PlantId, PlantOutput>, CloseStackError> {
        let (tx_back, rx) = oneshot::channel();
        let _ = self
            .tx
            .send(StackMessage::CloseStack {
                period_id: delivery_period,
                tx_back,
            })
            .await;

        rx.await.map_err(|_| CloseStackError(delivery_period))
    }

    async fn get_snapshot(
        &self,
    ) -> Result<HashMap<PlantId, PowerPlantPublicRepr>, GetSnashotError> {
        let (tx_back, rx) = oneshot::channel();
        let _ = self.tx.send(StackMessage::GetSnapshot(tx_back)).await;

        rx.await.map_err(|_| GetSnashotError)
    }

    async fn program_setpoint(&self, plant: PlantId, setpoint: isize) {
        let _ = self
            .tx
            .send(StackMessage::ProgramSetpoint(ProgramPlant {
                plant_id: plant,
                setpoint,
            }))
            .await;
    }
}

#[cfg(test)]
mockall::mock! {
    pub StackService {}

    impl StackService for StackService {
        fn open_stack(&self, delivery_period: DeliveryPeriodId) -> impl Future<Output = ()> + Send;

        fn close_stack(
        &self,
        delivery_period: DeliveryPeriodId,
    ) -> impl Future<Output = Result<HashMap<PlantId, PlantOutput>, CloseStackError>> + Send;

    fn program_setpoint(&self, plant: PlantId, setpoint: isize) -> impl Future<Output = ()> + Send;

    fn get_snapshot(
        &self,
    ) -> impl Future<Output = Result<HashMap<PlantId, PowerPlantPublicRepr>, GetSnashotError>> + Send;
    }

    impl Clone for StackService {
        fn clone(&self) -> Self;
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use tokio::sync::mpsc;

    use crate::{
        game::delivery_period::DeliveryPeriodId,
        plants::{
            models::{Service, StackService},
            stack::StackMessage,
            PlantId,
        },
    };

    #[tokio::test]
    async fn test_open_stack() {
        let (tx, _) = mpsc::channel(128);
        let service = Service::new(tx);

        let _ = service.open_stack(DeliveryPeriodId::from(0)).await;
    }

    #[tokio::test]
    async fn test_close_stack_ok() {
        let (tx, mut rx) = mpsc::channel(128);
        let service = Service::new(tx);

        tokio::spawn(async move {
            let Some(StackMessage::CloseStack {
                period_id: _,
                tx_back,
            }) = rx.recv().await
            else {
                unreachable!()
            };
            let _ = tx_back.send(HashMap::new());
        });

        let res = service.close_stack(DeliveryPeriodId::from(0)).await;
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_close_stack_err() {
        let (tx, mut rx) = mpsc::channel(128);
        let service = Service::new(tx);

        // Close receiving end to simulate err
        rx.close();

        let res = service.close_stack(DeliveryPeriodId::from(0)).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_get_snapshot_ok() {
        let (tx, mut rx) = mpsc::channel(128);
        let service = Service::new(tx);

        tokio::spawn(async move {
            let Some(StackMessage::GetSnapshot(tx_back)) = rx.recv().await else {
                unreachable!()
            };
            let _ = tx_back.send(HashMap::new());
        });

        let res = service.get_snapshot().await;
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_get_snapshot_err() {
        let (tx, mut rx) = mpsc::channel(128);
        let service = Service::new(tx);

        // Close receiving end to simulate err
        rx.close();

        let res = service.get_snapshot().await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_program_setpoint() {
        let (tx, _) = mpsc::channel(128);
        let service = Service::new(tx);

        let _ = service.program_setpoint(PlantId::default(), 0).await;
    }
}
