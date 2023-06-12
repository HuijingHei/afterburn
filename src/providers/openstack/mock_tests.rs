use crate::providers::openstack::OpenstackProviderNetwork;
use crate::providers::MetadataProvider;
use mockito;

#[test]
fn test_ssh_keys() {
    let mut server = mockito::Server::new();
    let mut provider = OpenstackProviderNetwork::try_new().unwrap();
    provider.client = provider.client.max_retries(0).mock_base_url(server.url());

    let key1 = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQCsXe6CfHl45kCIzMF92VhDf2NpBWUyS1+IiTtxm5a83mT9730Hb8xim7GYeJu47kiESw2DAN8vNJ/Irg0apZ217ah2rXXjPQuWYSXuEuap8yLBSjqw8exgqVj/kzW+YqmnHASxI13eoFDxTQQGzyqbqowvxu/5gQmDwBmNAa9bT809ziB/qmpS1mD6qyyFDpR23kUwu3TkgAbwMXBDoqK+pdwfaF9uo9XaLHNEH8lD5BZuG2BeDafm2o76DhNSo83MvcCPNXKLxu3BbX/FCMFO6O8RRqony4i91fEV1b8TbXrbJz1bwEYEnJRvmjnqI/389tQFeYvplXR2WdT9PCKyEAG+j8y6XgecIcdTqV/7gFfak1mp2S7mYHZDnXixsn3MjCP/cIxxJVDitKusnj1TdFqtSXl4tqGccbg/5Sqnt/EVSK4bGwwBxv/YmE0P9cbXLxuEVI0JYzgrQvC8TtUgd8kUu2jqi1/Yj9IWm3aFsl/hhh8YwYrv/gm8PV0TxkM= root@example1";
    let key2 = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDj6FBVgkTt7/DB93VVLk6304Nx7WUjLBJDSCh38zjCimHUpeo9uYDxflfu2N1CLtrSImIKBVP/JRy9g7K4zmRAH/wXw2UxYziX+hZoFIpbW3GmYQqhjx2lDvIRXJI7blhHhTUNWX5f10lFAYOLqA9J859AB1w7ND09+MS3jQgSazCx17h+QZ0qQ6kLSfnXw9PMUOE1Xba9hD1nYj14ryTVj9jrFPMFuUfXdb/G9lsDJ+cGvdE2/RMuPfDmEdo04zvZ5fQJJKvS7OyAuYev4Y+JC8MhEr756ITDZ17yq4BEMo/8rNPxZ5Von/8xnvry+8/2C3ep9rZyHtCwpRb6WT6TndV2ddXKhEIneyd1XiOcWPJguHj5vSoMN3mo8k2PvznGauvxBstvpjUSFLQu869/ZQwyMnbQi3wnkJk5CpLXePXn1J9njocJjt8+SKGijmmIAsmYosx8gmmu3H1mvq9Wi0qqWDITMm+J24AZBEPBhwVrjhLZb5MKxylF6JFJJBs= root@example2";
    let endpoints = maplit::btreemap! {
        "/latest/meta-data/public-keys" => "0=test1\n1=test2",
        "/latest/meta-data/public-keys/0/openssh-key" => key1,
        "/latest/meta-data/public-keys/1/openssh-key" => key2,
    };

    for (endpoint, body) in endpoints {
        server
            .mock("GET", endpoint)
            .with_status(200)
            .with_body(body)
            .create();
    }

    let keys = provider.ssh_keys().unwrap();
    assert_eq!(keys.len(), 2);
    assert_eq!(keys[0].options, None);
    assert_eq!(keys[0].comment, Some("root@example1".to_string()));

    assert_eq!(keys[1].options, None);
    assert_eq!(keys[1].comment, Some("root@example2".to_string()));

    server.reset();
    provider.ssh_keys().unwrap_err();
}

#[test]
fn test_ssh_keys_404_ok() {
    let mut server = mockito::Server::new();
    let mut provider = OpenstackProviderNetwork::try_new().unwrap();
    provider.client = provider.client.max_retries(0).mock_base_url(server.url());

    server
        .mock("GET", "/latest/meta-data/public-keys")
        .with_status(404)
        .create();
    let v = provider.ssh_keys().unwrap();
    assert_eq!(v.len(), 0);
    server.reset();
    provider.ssh_keys().unwrap_err();
}

#[test]
fn test_instance_uuid() {
    let mut server = mockito::Server::new();
    let mut provider = OpenstackProviderNetwork::try_new().unwrap();
    provider.client = provider.client.max_retries(0).mock_base_url(server.url());

    server
        .mock("GET", "/openstack/2012-08-10/meta_data.json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body_from_file(
            "tests/fixtures/openstack-metadata/openstack/2012-08-10/meta_data.json",
        )
        .create();

    server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/latest/meta-data/.*$".to_string()),
        )
        .with_status(404)
        .create();

    let v = provider.attributes().unwrap();
    assert_eq!(
        v.get("OPENSTACK_INSTANCE_UUID"),
        Some(&String::from("99dcf33b-6eb5-4acf-9abb-d81723e0c949"))
    );
    server.reset();
    provider.attributes().unwrap_err();
}

#[test]
fn test_instance_uuid_404_ok() {
    let mut server = mockito::Server::new();
    let mut provider = OpenstackProviderNetwork::try_new().unwrap();
    provider.client = provider.client.max_retries(0).mock_base_url(server.url());

    server
        .mock("GET", "/openstack/2012-08-10/meta_data.json")
        .with_status(404)
        .create();

    server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/latest/meta-data/.*$".to_string()),
        )
        .with_status(404)
        .create();

    let v = provider.attributes().unwrap();
    assert_eq!(v.len(), 0);
    server.reset();
    provider.attributes().unwrap_err();
}
