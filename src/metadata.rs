use crate::v1::MqttBroker;

impl MqttBroker {
    #[must_use] 
    pub fn namespace(&self) -> &str {
        self.metadata
            .namespace
            .as_ref()
            .expect("CRD without namespace")
    }
}
