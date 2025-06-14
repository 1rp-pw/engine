use serde_json::json;
use testcontainers::{clients::Cli, images::generic::GenericImage, Container};

pub struct PolicyEngineContainer<'a> {
    container: Container<'a, GenericImage>,
    port: u16,
}

impl<'a> PolicyEngineContainer<'a> {
    pub fn new(docker: &'a Cli) -> Self {
        let image = GenericImage::new("policy-engine", "latest")
            .with_exposed_port(3000)
            .with_env_var("FF_ENV_ID", "test-env")
            .with_env_var("FF_AGENT_ID", "test-agent")
            .with_env_var("FF_PROJECT_ID", "test-project")
            .with_wait_for(testcontainers::core::WaitFor::message_on_stdout("Server running"));

        let container = docker.run(image);
        let port = container.get_host_port_ipv4(3000);

        Self { container, port }
    }

    pub fn base_url(&self) -> String {
        format!("http://localhost:{}", self.port)
    }

    pub async fn evaluate_policy(&self, rule: &str, data: serde_json::Value, trace: bool) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let request_body = json!({
            "rule": rule,
            "data": data,
            "trace": trace
        });

        let response = client
            .post(&self.base_url())
            .json(&request_body)
            .send()
            .await?;

        let result = response.json::<serde_json::Value>().await?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use testcontainers::clients::Cli;

    #[tokio::test]
    async fn test_senior_discount_policy() {
        let docker = Cli::default();
        let policy_engine = PolicyEngineContainer::new(&docker);

        let rule = "A **Person** gets senior_discount if the __age__ of the **Person** is greater than or equal to 65.";
        let data = json!({
            "age": 70
        });

        let result = policy_engine
            .evaluate_policy(rule, data, true)
            .await
            .expect("Policy evaluation failed");

        // Check that the result contains senior_discount
        assert!(result.to_string().contains("senior_discount"));
        println!("Policy evaluation result: {}", serde_json::to_string_pretty(&result).unwrap());
    }

    #[tokio::test]
    async fn test_expedited_shipping_policy() {
        let docker = Cli::default();
        let policy_engine = PolicyEngineContainer::new(&docker);

        let rule = r#"An **Order** gets expedited_shipping if the __total__ of the **Order** is greater than 100 and the __membership_level__ of the **Customer** is in ["gold", "platinum"]."#;
        let data = json!({
            "total": 150.00,
            "Customer": {
                "membership_level": "gold"
            }
        });

        let result = policy_engine
            .evaluate_policy(rule, data, false)
            .await
            .expect("Policy evaluation failed");

        assert!(result.to_string().contains("expedited_shipping"));
    }

    #[tokio::test]
    async fn test_policy_rejection() {
        let docker = Cli::default();
        let policy_engine = PolicyEngineContainer::new(&docker);

        let rule = "A **Person** gets senior_discount if the __age__ of the **Person** is greater than or equal to 65.";
        let data = json!({
            "age": 45  // Below threshold
        });

        let result = policy_engine
            .evaluate_policy(rule, data, true)
            .await
            .expect("Policy evaluation failed");

        // Should not contain senior_discount for age < 65
        assert!(!result.to_string().contains("senior_discount") || 
                result.to_string().contains("false"));
    }
}