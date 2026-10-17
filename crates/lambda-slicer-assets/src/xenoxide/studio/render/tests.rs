use std::path::{Path, PathBuf};

use super::super::fixture::{self, BodyPart, Fixture, Mesh, Submodel, Texture};
use super::*;

/// The standard fixture with rendering data: two textures in two skin families, and
/// three body parts. `body` has one model, whose single mesh is a strip of four corners
/// and a fan of four; `head` has an empty model and one with a three-corner strip;
/// `gear` divides the body value by 2 among three models.
fn rendered() -> Fixture {
    let point = |x: f32, bone: u8| ([x, x + 1.0, x + 2.0], bone);
    let corner = |vertex: i16, normal: i16, s: i16, t: i16| [vertex, normal, s, t];
    let mut fixture = Fixture::standard();
    fixture.texture_data = vec![
        Texture::solid("skin.bmp", 4, 2, 7, [200, 100, 50]),
        Texture::solid("face.bmp", 2, 2, 3, [10, 20, 30]),
    ];
    fixture.skins = vec![vec![0, 1], vec![1, 1]];
    let blank = Submodel {
        name: "blank",
        vertices: Vec::new(),
        normals: Vec::new(),
        meshes: Vec::new(),
    };
    fixture.body_parts = vec![
        BodyPart {
            name: "body",
            base: 1,
            models: vec![Submodel {
                name: "torso",
                vertices: vec![
                    point(0.0, 0),
                    point(1.0, 1),
                    point(2.0, 2),
                    point(3.0, 1),
                    point(4.0, 0),
                ],
                normals: vec![point(0.5, 0), point(1.5, 2)],
                meshes: vec![Mesh {
                    skin_reference: 0,
                    commands: vec![
                        (
                            4,
                            vec![
                                corner(0, 0, 0, 0),
                                corner(1, 1, 4, 0),
                                corner(2, 0, 0, 2),
                                corner(3, 1, 4, 2),
                            ],
                        ),
                        (
                            -4,
                            vec![
                                corner(4, 1, 1, 1),
                                corner(0, 0, 2, 1),
                                corner(1, 1, 3, 1),
                                corner(2, 0, 3, 2),
                            ],
                        ),
                    ],
                    declared: None,
                }],
            }],
        },
        BodyPart {
            name: "head",
            base: 1,
            models: vec![
                blank.clone(),
                Submodel {
                    name: "face",
                    vertices: vec![point(5.0, 2), point(6.0, 2), point(7.0, 2)],
                    normals: vec![point(6.5, 2)],
                    meshes: vec![Mesh {
                        skin_reference: 1,
                        commands: vec![(
                            3,
                            vec![corner(0, 0, 0, 0), corner(1, 0, 2, 0), corner(2, 0, 0, 2)],
                        )],
                        declared: None,
                    }],
                },
            ],
        },
        BodyPart {
            name: "gear",
            base: 2,
            models: vec![blank.clone(), blank.clone(), blank],
        },
    ];
    fixture
}

fn read(fixture: &Fixture) -> Result<StudioRenderAsset, StudioError> {
    let built = fixture.build();
    let model = StudioModel::parse(&built.bytes)?.resolve(&[])?;
    let companion = fixture
        .textures_in_companion
        .then(|| fixture.build_texture_companion().bytes);
    StudioRenderAsset::parse(&model, &built.bytes, companion.as_deref())
}

/// Reads the rendering data of a built file after `mutate`
fn refused(mutate: impl FnOnce(&mut fixture::Built)) -> StudioError {
    let mut built = rendered().build();
    mutate(&mut built);
    let model = StudioModel::parse(&built.bytes)
        .unwrap()
        .resolve(&[])
        .unwrap();
    StudioRenderAsset::parse(&model, &built.bytes, None).unwrap_err()
}

#[test]
fn the_rendering_fixture_reads_completely() {
    let fixture = rendered();
    let asset = read(&fixture).unwrap();
    let textures = asset.textures();
    assert_eq!(textures.len(), 2);
    assert_eq!((textures[0].width, textures[0].height), (4, 2));
    assert_eq!(textures[0].pixels, vec![7; 8]);
    assert_eq!(textures[0].palette[7], [200, 100, 50]);
    assert_eq!(textures[1].name, "face.bmp");
    assert_eq!(asset.skin_families(), [vec![0, 1], vec![1, 1]]);

    let parts = asset.body_parts();
    assert_eq!(
        parts
            .iter()
            .map(|part| (part.name.as_str(), part.base, part.models.len()))
            .collect::<Vec<_>>(),
        [("body", 1, 1), ("head", 1, 2), ("gear", 2, 3)]
    );
    let torso = &parts[0].models[0];
    assert_eq!(
        torso.vertices[3],
        StudioPoint {
            position: [3.0, 4.0, 5.0],
            bone: 1
        }
    );
    assert_eq!(torso.normals[1].bone, 2);
    // A strip of four: (0, 1, 2) then (2, 1, 3); a fan of four: (4, 0, 1) then (4, 1, 2)
    let triangles: Vec<[usize; 3]> = torso.meshes[0]
        .triangles
        .iter()
        .map(|triangle| triangle.map(|corner| corner.vertex))
        .collect();
    assert_eq!(triangles, [[0, 1, 2], [2, 1, 3], [4, 0, 1], [4, 1, 2]]);
    assert_eq!(
        torso.meshes[0].triangles[1][2],
        StudioCorner {
            vertex: 3,
            normal: 1,
            s: 4,
            t: 2
        }
    );
    assert!(parts[1].models[0].meshes.is_empty());
    assert_eq!(parts[1].models[1].meshes[0].skin_reference, 1);

    let model = StudioModel::parse(&fixture.build().bytes)
        .unwrap()
        .resolve(&[])
        .unwrap();
    let identity = asset.identity();
    assert_eq!(identity.model, model.identity().combined);
    assert_eq!(identity.texture_companion, None);
    assert_eq!(
        identity.combined,
        StudioRenderIdentity::new(identity.model, None).combined
    );
}

/// The body value selects each part's model: `(body / base) % models`
#[test]
fn the_body_value_selects_each_parts_model() {
    let asset = read(&rendered()).unwrap();
    let selected = |body| {
        (0..3)
            .map(|part| asset.submodel(part, body))
            .collect::<Vec<_>>()
    };
    assert_eq!(selected(0), [0, 0, 0]);
    assert_eq!(selected(1), [0, 1, 0]);
    assert_eq!(selected(2), [0, 0, 1]);
    assert_eq!(selected(5), [0, 1, 2]);
    assert_eq!(selected(6), [0, 0, 0]);
}

/// A main file without textures needs its companion, and one with textures refuses
/// one; the companion is part of the rendering identity only
#[test]
fn the_texture_companion_is_a_rendering_dependency() {
    let mut fixture = rendered();
    fixture.textures_in_companion = true;
    let built = fixture.build();
    let model = StudioModel::parse(&built.bytes)
        .unwrap()
        .resolve(&[])
        .unwrap();
    assert!(model.texture_companion());
    assert_eq!(
        StudioRenderAsset::parse(&model, &built.bytes, None),
        Err(StudioError::MissingTextureCompanion)
    );
    let companion = fixture.build_texture_companion();
    let asset = StudioRenderAsset::parse(&model, &built.bytes, Some(&companion.bytes)).unwrap();
    assert_eq!(asset.textures(), read(&rendered()).unwrap().textures());
    assert_eq!(asset.identity().model, model.identity().combined);
    assert_eq!(
        asset.identity().texture_companion,
        Some(StudioDigest::of(&companion.bytes))
    );
    // Another companion, another rendering identity; the pose identity unchanged
    let mut recoloured = fixture.clone();
    recoloured.texture_data[0].palette[7] = [0, 0, 255];
    let other = StudioRenderAsset::parse(
        &model,
        &built.bytes,
        Some(&recoloured.build_texture_companion().bytes),
    )
    .unwrap();
    assert_ne!(other.identity().combined, asset.identity().combined);
    assert_eq!(other.identity().model, asset.identity().model);
    // A companion for a model with its own textures, and one without textures
    let own = rendered().build();
    let own_model = StudioModel::parse(&own.bytes)
        .unwrap()
        .resolve(&[])
        .unwrap();
    assert_eq!(
        StudioRenderAsset::parse(&own_model, &own.bytes, Some(&companion.bytes)),
        Err(StudioError::UnexpectedTextureCompanion)
    );
    let mut empty = fixture.clone();
    empty.texture_data.clear();
    empty.skins.clear();
    assert_eq!(
        StudioRenderAsset::parse(
            &model,
            &built.bytes,
            Some(&empty.build_texture_companion().bytes)
        ),
        Err(StudioError::InvalidTexture {
            index: 0,
            reason: "the texture companion has no textures"
        })
    );
    // The main model is no companion: its identifier is right, but it has no textures
    assert!(StudioRenderAsset::parse(&model, &built.bytes, Some(&built.bytes)).is_err());
}

/// Rendering data is read only from the file its model was read from
#[test]
fn rendering_data_belongs_to_its_model_file() {
    let built = rendered().build();
    let other = Fixture::standard().build();
    let other_model = StudioModel::parse(&other.bytes)
        .unwrap()
        .resolve(&[])
        .unwrap();
    assert_eq!(
        StudioRenderAsset::parse(&other_model, &built.bytes, None),
        Err(StudioError::RenderModelMismatch)
    );
}

#[test]
fn malformed_meshes_are_refused() {
    let mesh = |reason| StudioError::InvalidMesh {
        part: 0,
        model: 0,
        mesh: 0,
        reason,
    };
    let mut wrong_count = rendered();
    wrong_count.body_parts[0].models[0].meshes[0].declared = Some(5);
    assert_eq!(
        read(&wrong_count),
        Err(mesh("decoded triangles differ from the declared count"))
    );
    let mut outside = rendered();
    outside.body_parts[0].models[0].meshes[0].commands[0].1[2][0] = 5;
    assert_eq!(
        read(&outside),
        Err(mesh(
            "a corner lies outside the model's vertices or normals"
        ))
    );
    let mut negative = rendered();
    negative.body_parts[0].models[0].meshes[0].commands[1].1[0][1] = -1;
    assert_eq!(
        read(&negative),
        Err(mesh(
            "a corner lies outside the model's vertices or normals"
        ))
    );
    let mut short = rendered();
    short.body_parts[0].models[0].meshes[0].commands[0]
        .1
        .truncate(2);
    short.body_parts[0].models[0].meshes[0].commands[0].0 = 2;
    assert_eq!(
        read(&short),
        Err(mesh("a command has fewer than three corners"))
    );
    let mut unskinned = rendered();
    unskinned.body_parts[0].models[0].meshes[0].skin_reference = 2;
    assert_eq!(read(&unskinned), Err(mesh("skin reference")));
    // Commands reaching past the end of the file
    assert_eq!(
        refused(|built| {
            let at = built.meshes[0][0][0] + 4;
            let end = built.bytes.len() as i32;
            built.put_i32(at, end);
        }),
        mesh("commands lie outside the file")
    );
    // More triangles than a model may have
    let mut huge = rendered();
    let corners = (0..MAX_STUDIO_TRIANGLES + 3)
        .map(|index| [(index % 3) as i16, 0, 0, 0])
        .collect::<Vec<_>>();
    huge.body_parts[0].models[0].meshes[0].commands =
        vec![((MAX_STUDIO_TRIANGLES + 3) as i16, corners)];
    assert_eq!(
        read(&huge),
        Err(mesh("more triangles than the model may have"))
    );
}

#[test]
fn malformed_textures_skins_and_parts_are_refused() {
    let texture = |reason| StudioError::InvalidTexture { index: 1, reason };
    assert_eq!(
        refused(|built| built.put_i32(built.texture_table + 80 + 68, 0)),
        texture("dimensions")
    );
    assert_eq!(
        refused(|built| built.put_i32(
            built.texture_table + 80 + 72,
            MAX_STUDIO_TEXTURE_DIMENSION as i32 + 1
        )),
        texture("dimensions")
    );
    assert_eq!(
        refused(|built| {
            let end = built.bytes.len() as i32;
            built.put_i32(built.texture_table + 80 + 76, end - 10);
        }),
        texture("data lies outside the file")
    );
    let mut bad_skin = rendered();
    bad_skin.skins[1][0] = 2;
    assert_eq!(
        read(&bad_skin),
        Err(StudioError::InvalidSkin {
            family: 1,
            reference: 0
        })
    );
    let mut bad_bone = rendered();
    bad_bone.body_parts[1].models[1].vertices[1].1 = 3;
    assert_eq!(
        read(&bad_bone),
        Err(StudioError::InvalidSubmodel {
            part: 1,
            model: 1,
            reason: "a point names a bone the model does not have"
        })
    );
    let mut bad_base = rendered();
    bad_base.body_parts[2].base = 0;
    assert_eq!(
        read(&bad_base),
        Err(StudioError::InvalidBodyPart {
            index: 2,
            reason: "base"
        })
    );
    assert_eq!(
        refused(|built| built.put_i32(built.body_part_table + 76 + 64, 0)),
        StudioError::InvalidBodyPart {
            index: 1,
            reason: "no models"
        }
    );
    let mut nan = rendered();
    nan.body_parts[0].models[0].vertices[2].0[1] = f32::NAN;
    assert_eq!(
        read(&nan),
        Err(StudioError::NonFinite {
            what: "vertex",
            index: 2
        })
    );
    assert_eq!(
        refused(|built| built.put_i32(140 + 64, MAX_STUDIO_BODY_PARTS as i32 + 1)),
        StudioError::CountOverLimit {
            what: "body part",
            count: MAX_STUDIO_BODY_PARTS + 1,
            limit: MAX_STUDIO_BODY_PARTS
        }
    );
    assert_eq!(
        refused(|built| built.put_i32(built.body_part_table + 64, MAX_STUDIO_MODELS as i32 + 1)),
        StudioError::CountOverLimit {
            what: "model",
            count: MAX_STUDIO_MODELS + 1,
            limit: MAX_STUDIO_MODELS
        }
    );
    assert_eq!(
        refused(|built| built.put_i32(built.models[0][0] + 72, MAX_STUDIO_MESHES as i32 + 1)),
        StudioError::CountOverLimit {
            what: "mesh",
            count: MAX_STUDIO_MESHES + 1,
            limit: MAX_STUDIO_MESHES
        }
    );
    assert_eq!(
        refused(|built| built.put_i32(built.models[0][0] + 80, MAX_STUDIO_VERTICES as i32 + 1)),
        StudioError::CountOverLimit {
            what: "vertex",
            count: MAX_STUDIO_VERTICES + 1,
            limit: MAX_STUDIO_VERTICES
        }
    );
    assert_eq!(
        refused(|built| built.put_i32(140 + 40, MAX_STUDIO_TEXTURES as i32 + 1)),
        StudioError::CountOverLimit {
            what: "texture",
            count: MAX_STUDIO_TEXTURES + 1,
            limit: MAX_STUDIO_TEXTURES
        }
    );
    assert_eq!(
        refused(|built| built.put_i32(140 + 52, MAX_STUDIO_TEXTURES as i32 + 1)),
        StudioError::CountOverLimit {
            what: "skin reference",
            count: MAX_STUDIO_TEXTURES + 1,
            limit: MAX_STUDIO_TEXTURES
        }
    );
    assert_eq!(
        refused(|built| built.put_i32(140 + 52, -1)),
        StudioError::InvalidCount {
            what: "skin reference",
            count: -1
        }
    );
    // Skin references with no family to map them would leave every mesh untextured
    assert_eq!(
        refused(|built| built.put_i32(140 + 56, 0)),
        StudioError::InvalidCount {
            what: "skin family",
            count: 0
        }
    );
    assert_eq!(
        refused(|built| built.put_i32(140 + 56, MAX_STUDIO_SKIN_FAMILIES as i32 + 1)),
        StudioError::CountOverLimit {
            what: "skin family",
            count: MAX_STUDIO_SKIN_FAMILIES + 1,
            limit: MAX_STUDIO_SKIN_FAMILIES
        }
    );
}

/// The rendering data cut anywhere, the declared length made to match, is refused; the
/// pose data before it still reads
#[test]
fn rendering_data_cut_anywhere_is_refused() {
    let built = rendered().build();
    for length in built.rendering_start..built.bytes.len() {
        let bytes = built.truncated(length);
        let model = StudioModel::parse(&bytes).unwrap().resolve(&[]).unwrap();
        assert!(
            StudioRenderAsset::parse(&model, &bytes, None).is_err(),
            "cut at {length}"
        );
    }
}

/// No corruption of a single byte of rendering data panics
#[test]
fn corrupting_rendering_data_never_panics() {
    let built = rendered().build();
    for at in built.rendering_start..built.bytes.len() {
        for value in [0x00, 0x7f, 0x80, 0xff] {
            let mut bytes = built.bytes.clone();
            bytes[at] = value;
            let model = StudioModel::parse(&bytes).unwrap().resolve(&[]).unwrap();
            let _ = StudioRenderAsset::parse(&model, &bytes, None);
        }
    }
}

/// A local Half-Life installation (`XENOXIDE_GAME_DIRECTORY`, holding `valve`)
fn game_directory() -> PathBuf {
    PathBuf::from(
        std::env::var_os("XENOXIDE_GAME_DIRECTORY")
            .expect("XENOXIDE_GAME_DIRECTORY must point to a Half-Life installation"),
    )
}

/// A stock model's pose and rendering data, with its sequence groups (named relative to
/// `mod_directory` with Windows separators) and its texture companion (`<name>t.mdl`
/// beside it) when its textures are there
fn load_rendering(
    mod_directory: &Path,
    path: &str,
) -> Result<(StudioModel, StudioRenderAsset), StudioError> {
    let full = mod_directory.join(path);
    let bytes = std::fs::read(&full).expect("model file");
    let unresolved = StudioModel::parse(&bytes)?;
    let files: Vec<(usize, Vec<u8>)> = unresolved
        .dependencies()
        .iter()
        .filter_map(|dependency| {
            let relative = dependency.name.replace('\\', "/");
            Some((
                dependency.index,
                std::fs::read(mod_directory.join(relative)).ok()?,
            ))
        })
        .collect();
    let borrowed: Vec<(usize, &[u8])> = files
        .iter()
        .map(|(index, bytes)| (*index, bytes.as_slice()))
        .collect();
    let model = unresolved.resolve(&borrowed)?;
    let companion = if model.texture_companion() {
        let stem = full.file_stem().unwrap().to_string_lossy().to_string();
        std::fs::read(full.with_file_name(format!("{stem}t.mdl"))).ok()
    } else {
        None
    };
    let asset = StudioRenderAsset::parse(&model, &bytes, companion.as_deref())?;
    Ok((model, asset))
}

/// The rendering data of every stock main model in `valve/models` and its player
/// models reads, except the two the pose reader refuses
#[test]
#[ignore = "requires XENOXIDE_GAME_DIRECTORY"]
fn every_stock_models_rendering_data_reads() {
    let valve = game_directory().join("valve");
    let mut paths = Vec::new();
    for directory in ["models"] {
        for entry in std::fs::read_dir(valve.join(directory)).unwrap() {
            let name = entry.unwrap().file_name().into_string().unwrap();
            if name.to_ascii_lowercase().ends_with(".mdl") {
                paths.push(format!("{directory}/{name}"));
            }
        }
    }
    for entry in std::fs::read_dir(valve.join("models/player")).unwrap() {
        let name = entry.unwrap().file_name().into_string().unwrap();
        let model = format!("models/player/{name}/{name}.mdl");
        if valve.join(&model).exists() {
            paths.push(model);
        }
    }
    paths.sort();
    let (mut read, mut failures) = (0, Vec::new());
    for path in &paths {
        let bytes = std::fs::read(valve.join(path)).unwrap();
        let header = |index: usize| i32_at(&bytes, 140 + index * 4);
        if bytes.get(0..4) != Some(b"IDST") || (header(0) == 0 && header(6) == 0) {
            continue;
        }
        match load_rendering(&valve, path) {
            Ok(_) => read += 1,
            Err(StudioError::UnsupportedVersion { version: 9, .. })
                if path.ends_with("doctor.mdl") => {}
            Err(StudioError::MissingDependency { .. }) if path.ends_with("strooper.mdl") => {}
            Err(error) => failures.push(format!("{path}: {error}")),
        }
    }
    println!("read {read}");
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

/// The stock player model's rendering data, with its texture companion: 28 textures, the
/// first chrome and flat-shaded, one skin family, a headless body and two heads
#[test]
#[ignore = "requires XENOXIDE_GAME_DIRECTORY"]
fn the_stock_player_models_rendering_data() {
    let valve = game_directory().join("valve");
    let (model, asset) = load_rendering(&valve, "models/player.mdl").unwrap();
    assert!(model.texture_companion());
    let identity = asset.identity();
    assert_eq!(identity.model, model.identity().combined);
    assert_eq!(
        identity.texture_companion.unwrap().to_string(),
        "d9ff51d4d4105774704dd058036d1ab8f394cf3a86eab226cbcb5de6eb47a34f"
    );
    assert_eq!(
        identity.combined.to_string(),
        "a856f081fd558f6d1e7b77ce95659e15e61d002291cdeb7643eed388997b1fe6"
    );
    let textures = asset.textures();
    assert_eq!(textures.len(), 28);
    assert_eq!(
        (
            textures[0].name.as_str(),
            textures[0].width,
            textures[0].height,
            textures[0].flags
        ),
        ("PLAYER_Chrome1.bmp", 64, 64, 3)
    );
    assert!(textures[1..].iter().all(|texture| texture.flags == 0));
    assert_eq!(asset.skin_families(), [(0..28).collect::<Vec<_>>()]);
    let models: Vec<_> = asset
        .body_parts()
        .iter()
        .flat_map(|part| {
            part.models.iter().map(move |submodel| {
                (
                    part.name.as_str(),
                    part.base,
                    submodel.name.as_str(),
                    submodel.vertices.len(),
                    submodel.normals.len(),
                    submodel.meshes.len(),
                    submodel
                        .meshes
                        .iter()
                        .map(|mesh| mesh.triangles.len())
                        .sum::<usize>(),
                )
            })
        })
        .collect();
    assert_eq!(
        models,
        [
            (
                "body",
                1,
                "player_template_biped(No_Head)1",
                255,
                507,
                18,
                486
            ),
            (
                "head",
                1,
                "player_template_biped(Gordon_Head)1",
                92,
                113,
                5,
                153
            ),
            ("head", 1, "player_template_biped(Helmet)1", 52, 88, 6, 94),
        ]
    );
}

/// The whole asset's triangles, vertices and normals, and texture pixels are budgeted
/// across every body part, model and texture, so expansion is bounded however the
/// per-model limits multiply. Each budget is met exactly by one asset and exceeded by
/// one more unit in another.
#[test]
fn the_whole_assets_rendering_data_is_budgeted() {
    // The rendering fixture already has 5 triangles, 11 vertices and normals, and 12
    // texture pixels
    let strip = |triangles: usize| Submodel {
        name: "strip",
        vertices: vec![([0.0; 3], 0); 3],
        normals: vec![([0.0, 0.0, 1.0], 0)],
        meshes: vec![Mesh {
            skin_reference: 0,
            commands: vec![(
                triangles as i16 + 2,
                (0..triangles + 2)
                    .map(|corner| [(corner % 3) as i16, 0, 0, 0])
                    .collect(),
            )],
            declared: None,
        }],
    };
    let with_strips = |last: usize| {
        let mut fixture = rendered();
        for models in [vec![strip(16_384)], vec![strip(last)]] {
            fixture.body_parts.push(BodyPart {
                name: "strips",
                base: 1,
                models,
            });
        }
        fixture
    };
    // 5 + 16384 + 16379 = 32768
    assert!(read(&with_strips(16_379)).is_ok());
    assert_eq!(
        read(&with_strips(16_380)).err(),
        Some(StudioError::RenderBudgetExceeded {
            what: "triangles",
            limit: MAX_STUDIO_ASSET_TRIANGLES
        })
    );

    let cloud = |vertices: usize, normals: usize| Submodel {
        name: "cloud",
        vertices: vec![([1.0; 3], 0); vertices],
        normals: vec![([0.0, 0.0, 1.0], 0); normals],
        meshes: Vec::new(),
    };
    let with_clouds = |last_normals: usize| {
        let mut fixture = rendered();
        let mut models = vec![cloud(2048, 2048); 15];
        models.push(cloud(2048, last_normals));
        fixture.body_parts.push(BodyPart {
            name: "clouds",
            base: 1,
            models,
        });
        fixture
    };
    // 11 + 15 × 4096 + 2048 + 2037 = 65536
    assert!(read(&with_clouds(2037)).is_ok());
    assert_eq!(
        read(&with_clouds(2038)).err(),
        Some(StudioError::RenderBudgetExceeded {
            what: "vertices and normals",
            limit: MAX_STUDIO_ASSET_POINTS
        })
    );

    let with_textures = |last_height: i32| {
        let mut fixture = rendered();
        for _ in 0..3 {
            fixture
                .texture_data
                .push(Texture::solid("big.bmp", 1024, 1024, 1, [1, 1, 1]));
        }
        fixture
            .texture_data
            .push(Texture::solid("tall.bmp", 1024, 1023, 1, [1, 1, 1]));
        fixture
            .texture_data
            .push(Texture::solid("last.bmp", 4, last_height, 1, [1, 1, 1]));
        fixture
    };
    // 12 + 3 × 1048576 + 1047552 + 1012 = 4194304
    assert!(read(&with_textures(253)).is_ok());
    assert_eq!(
        read(&with_textures(254)).err(),
        Some(StudioError::RenderBudgetExceeded {
            what: "texture pixels",
            limit: MAX_STUDIO_ASSET_TEXTURE_PIXELS
        })
    );
}
