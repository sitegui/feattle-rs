use feattle::*;

feattle_enum! {
    CalculateMoneySupply {
        High,
        Low,
    }
}

feattle_enum! {
    CalibratePersonalityMatrix {
        Rows,
        Columns,
        Diagonals,
        AntiDiagonals,
    }
}

feattle_enum! {
    MapInfluenceAttributes {
        Bias,
        Linear,
        Square,
    }
}

feattles! {
    Features {
        extrude_mesh_terrain: bool = true,
        /// A short description
        balance_domestic_coefficients: i32 = 2,
        /// A longer, complete description, bringing attention to contentious issues surrounding
        /// this configuration and what could go wrong if misused.
        invert_career_ladder: f64 = 3.6,
        calculate_money_supply: CalculateMoneySupply = CalculateMoneySupply::High,
        normalize_social_network: String = "normal".to_owned(),
        adjust_emotional_weights: std::collections::BTreeSet<i32>,
        calibrate_personality_matrix: std::collections::BTreeSet<CalibratePersonalityMatrix>,
        insert_chaos_generator: std::collections::BTreeSet<String>,
        map_influence_attributes: std::collections::BTreeMap<MapInfluenceAttributes, i32>,
        assign_mimic_propagation: std::collections::BTreeMap<String, i32>,
        blah: Option<Vec<i32>>,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    use feattle_sync::persist::Disk;
    use std::sync::Arc;

    let disk_storage = Disk::new("data")?;
    let features = Arc::new(Features::new(disk_storage));
    BackgroundSync::new(&features).spawn();
    dbg!(features.last_reload());
    dbg!(features.current_values());
    for def in features.definitions() {
        println!("{} = {}", def.key, serde_json::to_string(&def.format)?);
    }

    let panel = AdminPanel::new(features.clone(), "gui".to_owned());
    warp::serve(panel.warp_filter())
        .run(([127, 0, 0, 1], 3030))
        .await;

    Ok(())
}
