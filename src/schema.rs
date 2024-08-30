// @generated automatically by Diesel CLI.

diesel::table! {
    sccache_clusters (id) {
        id -> Uuid,
        #[max_length = 256]
        client_id -> Varchar,
        #[max_length = 256]
        endpoint -> Varchar,
        #[max_length = 256]
        token -> Varchar,
        #[max_length = 256]
        region -> Varchar,
        #[max_length = 256]
        ec2_instance_type -> Varchar,
    }
}

diesel::table! {
    sccache_instances (id) {
        id -> Uuid,
        cluster_id -> Uuid,
        #[max_length = 256]
        instance_id -> Varchar,
        #[max_length = 256]
        instance_type -> Varchar,
    }
}

diesel::table! {
    test_table (test_schema_key) {
        test_schema_key -> Varchar,
    }
}

diesel::joinable!(sccache_instances -> sccache_clusters (cluster_id));

diesel::allow_tables_to_appear_in_same_query!(
    sccache_clusters,
    sccache_instances,
    test_table,
);
