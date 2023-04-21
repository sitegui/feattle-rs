use axum::Server;
use feattle::*;
use rusoto_core::Region;
use rusoto_s3::S3Client;
use std::collections::BTreeMap;
use std::env;
use std::error::Error;
use std::sync::Arc;
use tokio::time::Duration;
use uuid::Uuid;

// Declare the struct that will gather all the feature toggles.
// Each field is turned into a method and allows a default value to be provided.
feattles! {
    struct SimulationToggles {
        /// Activates extruding the mesh terrain, usually in the minor inertia axis.
        extrude_mesh_terrain: bool,
        /// The domestic module being always present, requires some balancing.
        balance_domestic_coefficients: u8 = 17,
        /// When to pause the bucolic routine to wonder about future capital availability
        calculate_money_supply: CalculateMoneySupply = CalculateMoneySupply::EveryNowAndThen,
        /// Some entities will respond to some links only
        map_influence_attributes: BTreeMap<Uuid, MapInfluenceAttributes> = default_map_influence_attributes(),
        /// A boolean, or not, for more chaos.
        iterate_chaos_array: Option<bool>,
    }
}

// Declare the enums that can be used as types for feattles
feattle_enum! {
    enum CalculateMoneySupply {
        Always,
        EveryNowAndThen,
        MostlyAvoid,
    }
}

feattle_enum! {
    #[allow(clippy::enum_variant_names)]
    enum MapInfluenceAttributes {
        FriendsOnly,
        FamilyOnly,
        PetsOnly,
        ImaginaryDragonsOnly,
    }
}

/// Provide the default value for `map_influence_attributes`
fn default_map_influence_attributes() -> BTreeMap<Uuid, MapInfluenceAttributes> {
    let mut value = BTreeMap::new();
    value.insert(
        Uuid::parse_str("4d57d909-c87d-4e89-b40d-236d23b59e82").unwrap(),
        MapInfluenceAttributes::FamilyOnly,
    );
    value.insert(
        Uuid::parse_str("aa8d90e6-8745-4c89-a48b-b8499347fe31").unwrap(),
        MapInfluenceAttributes::ImaginaryDragonsOnly,
    );
    value
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load configurations from `.env` file, please copy this template and fill it in:
    // ```
    // AWS_ACCESS_KEY_ID=
    // AWS_SECRET_ACCESS_KEY=
    // AWS_REGION=
    // S3_BUCKET=
    // S3_KEY_PREFIX=
    // ADMIN_PANEL_LABEL=
    // ```
    dotenv::dotenv()?;
    let s3_bucket = env::var("S3_BUCKET").unwrap();
    let s3_key_prefix = env::var("S3_KEY_PREFIX").unwrap();
    let admin_panel_label = env::var("ADMIN_PANEL_LABEL").unwrap();
    env_logger::init();

    // Create the S3 persistence
    let client = S3Client::new(Region::default());
    let s3_storage = S3::new(client, s3_bucket, s3_key_prefix);

    // Create the instance
    let features = Arc::new(SimulationToggles::new(s3_storage));

    // Sync in the background
    BackgroundSync::new(&features).spawn();

    // Create the admin panel
    let panel = Arc::new(AdminPanel::new(features.clone(), admin_panel_label));

    // Serve the admin panel with `warp`
    tokio::spawn(run_warp_server(panel.clone(), ([127, 0, 0, 1], 3030)));

    // Serve the admin panel with `axum`
    let router = axum_router(panel);
    tokio::spawn(Server::bind(&([127, 0, 0, 1], 3031).into()).serve(router.into_make_service()));

    println!("Admin UI available in http://127.0.0.1:3030");

    loop {
        println!("--- Values are ---");
        println!(
            "extrude_mesh_terrain = {:?}",
            features.extrude_mesh_terrain()
        );
        println!(
            "balance_domestic_coefficients = {:?}",
            features.balance_domestic_coefficients()
        );
        println!(
            "calculate_money_supply = {:?}",
            features.calculate_money_supply()
        );
        println!(
            "map_influence_attributes = {:?}",
            features.map_influence_attributes()
        );
        println!("iterate_chaos_array = {:?}", features.iterate_chaos_array());
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
