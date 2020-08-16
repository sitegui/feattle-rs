use feattle::*;
use std::thread::sleep;
use std::time::Duration;

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
    }
}

pub mod gui {
    use feattle_core::__internal;

    pub struct Gui<P>(__internal::FeattlesImpl<P, __Features>);
    pub struct __Features {
        extrude_mesh_terrain: __internal::Feature<bool>,
    }

    impl __internal::FeaturesStruct for __Features {
        fn update(
            &mut self,
            key: &str,
            value: &__internal::CurrentValue,
        ) -> Result<(), __internal::FromJsonError> {
            match key {
                "extrude_mesh_terrain" => self.extrude_mesh_terrain.update(value),
                _ => unreachable!(),
            }
        }
    }

    impl<P: __internal::Persist> __internal::Feattles<P> for Gui<P> {
        type FeatureStruct = __Features;

        fn _read(
            &self,
        ) -> __internal::RwLockReadGuard<'_, __internal::InnerFeattles<Self::FeatureStruct>>
        {
            self.0.inner_feattles.read()
        }

        fn _write(
            &self,
        ) -> __internal::RwLockWriteGuard<'_, __internal::InnerFeattles<Self::FeatureStruct>>
        {
            self.0.inner_feattles.write()
        }

        fn new(persistence: P) -> Self {
            Gui(__internal::FeattlesImpl::new(
                persistence,
                __Features {
                    extrude_mesh_terrain: __internal::Feature::new(
                        "extrude_mesh_terrain",
                        "",
                        true,
                    ),
                },
            ))
        }

        fn persistence(&self) -> &P {
            &self.0.persistence
        }

        fn keys(&self) -> &'static [&'static str] {
            &["extrude_mesh_terrain"]
        }

        fn definition(&self, key: &str) -> Option<__internal::FeatureDefinition> {
            let inner = self._read();
            match key {
                "extrude_mesh_terrain" => {
                    Some(inner.feattles_struct.extrude_mesh_terrain.definition())
                }
                _ => None,
            }
        }
    }

    impl<P: __internal::Persist> Gui<P> {
        pub fn extrude_mesh_terrain(&self) -> __internal::MappedRwLockReadGuard<bool> {
            __internal::RwLockReadGuard::map(self.0.inner_feattles.read(), |inner| {
                &inner.feattles_struct.extrude_mesh_terrain.value
            })
        }
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
    dbg!(features.definitions());
    dbg!(features.last_reload());
    dbg!(features.current_values());

    sleep(Duration::from_secs(1));
    dbg!(features.last_reload());
    dbg!(features.current_values());

    if *features.extrude_mesh_terrain() {
        println!("OK");
    }

    warp::serve(ui(features.clone()))
        .run(([127, 0, 0, 1], 3030))
        .await;

    Ok(())
}
