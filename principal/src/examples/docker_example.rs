use std::collections::HashMap;
use tokio::time::{sleep, Duration};

use crate::providers::local::models::request::container::{
    LocalContainer, CreateContainerRequest, PortMapping, VolumeMount,
};

/// Example demonstrating real Docker integration
pub async fn run_docker_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Infralink Real Docker Integration Example ===");

    // Check Docker availability
    check_docker_availability().await?;

    // Run container lifecycle examples
    demo_basic_container_lifecycle().await?;
    demo_container_with_ports().await?;
    demo_container_with_volumes().await?;
    demo_container_management().await?;
    demo_container_monitoring().await?;

    println!("\n✅ Docker integration examples completed successfully!");
    Ok(())
}

async fn check_docker_availability() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n1. Checking Docker availability...");

    match LocalContainer::list().await {
        Ok(containers) => {
            println!("✅ Docker is available");
            println!("   Found {} existing containers", containers.len());
        }
        Err(e) => {
            println!("❌ Docker not available: {}", e);
            println!("   Please ensure Docker is installed and running");
            return Err(e);
        }
    }

    Ok(())
}

async fn demo_basic_container_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n2. Basic Container Lifecycle");

    // Create a simple container
    let create_request = CreateContainerRequest {
        name: Some("infralink-demo-basic".to_string()),
        image: "alpine:latest".to_string(),
        command: Some(vec!["sleep".to_string(), "30".to_string()]),
        env: Some(vec![
            "DEMO_VAR=hello".to_string(),
            "INFRALINK_ENV=test".to_string(),
        ]),
        labels: Some({
            let mut labels = HashMap::new();
            labels.insert("demo.type".to_string(), "basic".to_string());
            labels.insert("demo.purpose".to_string(), "lifecycle".to_string());
            labels
        }),
        ..Default::default()
    };

    let container_id = LocalContainer::create(create_request).await?;
    println!("✅ Created container: {}", container_id);

    // Start the container
    LocalContainer::start(&container_id).await?;
    println!("✅ Started container");

    // Give it a moment to run
    sleep(Duration::from_secs(2)).await;

    // Inspect the container
    let container_info = LocalContainer::inspect(&container_id).await?;
    println!("✅ Container status: {} ({})", container_info.state, container_info.status);

    // Get logs
    let logs = LocalContainer::logs(&container_id, false, Some("10".to_string())).await?;
    println!("✅ Container logs (last 10 lines):");
    if logs.trim().is_empty() {
        println!("   (no output - container is sleeping)");
    } else {
        for line in logs.lines().take(5) {
            println!("   {}", line);
        }
    }

    // Execute a command
    let exec_output = LocalContainer::exec(&container_id, vec![
        "sh".to_string(),
        "-c".to_string(),
        "echo 'Hello from inside container'; ps aux".to_string(),
    ]).await?;
    println!("✅ Exec output:");
    for line in exec_output.lines().take(5) {
        println!("   {}", line);
    }

    // Stop and remove the container
    LocalContainer::stop(&container_id).await?;
    println!("✅ Stopped container");

    LocalContainer::remove(&container_id).await?;
    println!("✅ Removed container");

    Ok(())
}

async fn demo_container_with_ports() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n3. Container with Port Mapping");

    // Create a container with nginx
    let create_request = CreateContainerRequest {
        name: Some("infralink-demo-nginx".to_string()),
        image: "nginx:alpine".to_string(),
        ports: Some(vec![PortMapping {
            host_port: 8080,
            container_port: 80,
            protocol: "tcp".to_string(),
        }]),
        labels: Some({
            let mut labels = HashMap::new();
            labels.insert("demo.type".to_string(), "web-server".to_string());
            labels
        }),
        ..Default::default()
    };

    let container_id = LocalContainer::create(create_request).await?;
    println!("✅ Created nginx container: {}", container_id);

    // Start the container
    LocalContainer::start(&container_id).await?;
    println!("✅ Started nginx container");

    // Give nginx time to start
    sleep(Duration::from_secs(3)).await;

    // Check if it's running
    let container_info = LocalContainer::inspect(&container_id).await?;
    println!("✅ Nginx status: {} ({})", container_info.state, container_info.status);
    println!("   Port mappings: {:?}", container_info.ports);

    // Test the web server (basic check)
    match tokio::time::timeout(Duration::from_secs(5), test_http_endpoint("http://localhost:8080")).await {
        Ok(Ok(_)) => println!("✅ Nginx is responding on port 8080"),
        Ok(Err(e)) => println!("⚠️  Nginx might not be ready yet: {}", e),
        Err(_) => println!("⚠️  Timeout testing nginx endpoint"),
    }

    // Get nginx logs
    let logs = LocalContainer::logs(&container_id, false, Some("5".to_string())).await?;
    println!("✅ Nginx logs:");
    for line in logs.lines().take(3) {
        println!("   {}", line);
    }

    // Clean up
    LocalContainer::stop(&container_id).await?;
    LocalContainer::remove(&container_id).await?;
    println!("✅ Cleaned up nginx container");

    Ok(())
}

async fn demo_container_with_volumes() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n4. Container with Volume Mounts");

    // Create a temporary directory for demo
    let temp_dir = "/tmp/infralink-demo";
    tokio::fs::create_dir_all(temp_dir).await.unwrap_or_default();
    
    // Write a test file
    tokio::fs::write(format!("{}/test.txt", temp_dir), "Hello from host!").await?;

    let create_request = CreateContainerRequest {
        name: Some("infralink-demo-volumes".to_string()),
        image: "alpine:latest".to_string(),
        command: Some(vec![
            "sh".to_string(),
            "-c".to_string(),
            "ls -la /mnt/host && cat /mnt/host/test.txt && echo 'Container was here' > /mnt/host/container.txt && sleep 10".to_string(),
        ]),
        volumes: Some(vec![VolumeMount {
            source: temp_dir.to_string(),
            destination: "/mnt/host".to_string(),
            mode: "rw".to_string(),
        }]),
        labels: Some({
            let mut labels = HashMap::new();
            labels.insert("demo.type".to_string(), "volume-test".to_string());
            labels
        }),
        ..Default::default()
    };

    let container_id = LocalContainer::create(create_request).await?;
    println!("✅ Created container with volume mount: {}", container_id);

    // Start and wait for completion
    LocalContainer::start(&container_id).await?;
    println!("✅ Started container");

    // Give it time to run
    sleep(Duration::from_secs(3)).await;

    // Get logs to see volume interaction
    let logs = LocalContainer::logs(&container_id, false, None).await?;
    println!("✅ Volume interaction logs:");
    for line in logs.lines().take(10) {
        println!("   {}", line);
    }

    // Check if container created the file
    match tokio::fs::read_to_string(format!("{}/container.txt", temp_dir)).await {
        Ok(content) => println!("✅ Container created file with content: {}", content.trim()),
        Err(_) => println!("⚠️  Container file not found (container might still be running)"),
    }

    // Clean up
    LocalContainer::stop(&container_id).await?;
    LocalContainer::remove(&container_id).await?;
    println!("✅ Cleaned up volume container");

    // Clean up temp directory
    let _ = tokio::fs::remove_dir_all(temp_dir).await;

    Ok(())
}

async fn demo_container_management() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n5. Container Management Operations");

    // List all containers
    let all_containers = LocalContainer::list().await?;
    println!("✅ Total containers (including stopped): {}", all_containers.len());

    // Show infralink-managed containers
    let infralink_containers: Vec<_> = all_containers.iter()
        .filter(|c| c.labels.get("infralink.managed").map_or(false, |v| v == "true"))
        .collect();
    
    println!("   Infralink-managed containers: {}", infralink_containers.len());
    for container in infralink_containers {
        println!("     - {} ({}) [{}]", container.name, container.id[..12].to_string(), container.state);
    }

    // Create multiple containers for management demo
    let mut demo_containers = Vec::new();
    
    for i in 1..=3 {
        let create_request = CreateContainerRequest {
            name: Some(format!("infralink-mgmt-demo-{}", i)),
            image: "alpine:latest".to_string(),
            command: Some(vec!["sleep".to_string(), "60".to_string()]),
            labels: Some({
                let mut labels = HashMap::new();
                labels.insert("demo.batch".to_string(), "management".to_string());
                labels.insert("demo.number".to_string(), i.to_string());
                labels
            }),
            ..Default::default()
        };

        let container_id = LocalContainer::create(create_request).await?;
        LocalContainer::start(&container_id).await?;
        println!("✅ Created and started demo container {}: {}", i, container_id[..12].to_string());
        demo_containers.push(container_id);
    }

    // Give containers time to start
    sleep(Duration::from_secs(2)).await;

    // Demonstrate batch operations
    println!("✅ Stopping all demo containers...");
    for container_id in &demo_containers {
        LocalContainer::stop(container_id).await?;
        println!("   Stopped: {}", container_id[..12].to_string());
    }

    println!("✅ Removing all demo containers...");
    for container_id in demo_containers {
        LocalContainer::remove(&container_id).await?;
        println!("   Removed: {}", container_id[..12].to_string());
    }

    Ok(())
}

async fn demo_container_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n6. Container Monitoring and Stats");

    // Create a container that does some work
    let create_request = CreateContainerRequest {
        name: Some("infralink-demo-stats".to_string()),
        image: "alpine:latest".to_string(),
        command: Some(vec![
            "sh".to_string(),
            "-c".to_string(),
            "while true; do echo 'Working...'; dd if=/dev/zero of=/tmp/test bs=1M count=10 2>/dev/null; sleep 2; done".to_string(),
        ]),
        memory_limit: Some(128 * 1024 * 1024), // 128MB limit
        labels: Some({
            let mut labels = HashMap::new();
            labels.insert("demo.type".to_string(), "stats".to_string());
            labels
        }),
        ..Default::default()
    };

    let container_id = LocalContainer::create(create_request).await?;
    LocalContainer::start(&container_id).await?;
    println!("✅ Created stats demo container: {}", container_id[..12].to_string());

    // Give it time to do some work
    sleep(Duration::from_secs(5)).await;

    // Get container stats
    match LocalContainer::stats(&container_id).await {
        Ok(stats) => {
            println!("✅ Container stats:");
            println!("   CPU Usage: {:.2}%", stats.cpu_usage_percent);
            println!("   Memory Usage: {:.2} MB ({:.1}%)", 
                     stats.memory_usage_bytes as f64 / 1024.0 / 1024.0,
                     stats.memory_usage_percent);
            println!("   Memory Limit: {:.2} MB", 
                     stats.memory_limit_bytes as f64 / 1024.0 / 1024.0);
            println!("   Network RX: {} bytes", stats.network_rx_bytes);
            println!("   Network TX: {} bytes", stats.network_tx_bytes);
            println!("   Block Read: {} bytes", stats.block_read_bytes);
            println!("   Block Write: {} bytes", stats.block_write_bytes);
            println!("   PIDs: {}", stats.pids);
        }
        Err(e) => {
            println!("⚠️  Could not get stats: {}", e);
        }
    }

    // Get recent logs
    let logs = LocalContainer::logs(&container_id, false, Some("5".to_string())).await?;
    println!("✅ Recent activity logs:");
    for line in logs.lines().take(3) {
        println!("   {}", line);
    }

    // Clean up
    LocalContainer::stop(&container_id).await?;
    LocalContainer::remove(&container_id).await?;
    println!("✅ Cleaned up stats container");

    Ok(())
}

async fn test_http_endpoint(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let response = reqwest::get(url).await?;
    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("HTTP error: {}", response.status()).into())
    }
}

impl Default for CreateContainerRequest {
    fn default() -> Self {
        Self {
            name: None,
            image: String::new(),
            command: None,
            env: None,
            ports: None,
            volumes: None,
            labels: None,
            restart_policy: None,
            working_dir: None,
            user: None,
            memory_limit: None,
            cpu_limit: None,
            auto_remove: None,
            detach: None,
        }
    }
}