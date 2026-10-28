//! Authored admission and prepared-backing witnesses; native capture remains separate.
#![cfg(sm_verify_dll)]
#![forbid(unsafe_code)]

use crate::game::scene::{
    records as sc,
    samus_body::Half::{self, Bottom, Top},
    samus_content::{
        AllocationDeclaration, AllocationId, ContentDescription,
        Registration::{self, *},
        RequestBinding,
        Role::{self, *},
        RoleDeclaration, ValidatedContent,
    },
    samus_position_tiles::TileRequest,
    state::{
        ByteLanes, DisplayError, SamusHalf, SourceSpan, TileDestination, TileTransfer,
        TransferSource,
        WordOrder::{self, Big, Little},
    },
};
use std::sync::Arc;

type RoleInput = (Role, Registration, u8, usize, WordOrder);
type TransferInput = (u16, u32, [u8; 2], usize, usize);
type RequestInput = (
    (Half, u16, u8, u32),
    TransferInput,
    TransferInput,
    TileDestination,
);

// Movies cannot supply independent full allocations and explicit portable request identities.
fn description(
    rows: [RoleInput; 12],
    requests: [RequestInput; 2],
    gameplay: [u8; 32],
    revision: sc::Revision,
    extent: usize,
    bytes: [u8; 8],
) -> ContentDescription {
    let allocations = rows
        .iter()
        .map(|&(_, _, tag, _, _)| {
            let identity = AllocationId([tag; 32]);
            (
                identity,
                Arc::new(AllocationDeclaration {
                    identity,
                    revision,
                    extent,
                    bytes: Arc::from(bytes),
                }),
            )
        })
        .collect();
    let roles = rows
        .into_iter()
        .map(|(role, registration, tag, mapped_base, native_order)| {
            (
                (role, registration),
                RoleDeclaration {
                    allocation: AllocationId([tag; 32]),
                    mapped_base,
                    native_order,
                },
            )
        })
        .collect();
    let requests = requests
        .into_iter()
        .map(
            |((half, pointer, enabled, descriptor), first, second, destination)| {
                let transfer = |(count, id, pair, words, offset): TransferInput| TileTransfer {
                    count,
                    destination,
                    source: TransferSource::Asset(SourceSpan {
                        asset: sc::AssetRef {
                            id: sc::AssetId(id),
                            revision: sc::Revision([0x41; 32]),
                        },
                        bytes: pair.repeat(words).into(),
                        offset,
                    }),
                };
                RequestBinding {
                    half,
                    request: TileRequest {
                        pointer,
                        half: SamusHalf {
                            enabled,
                            descriptor: sc::AssetRef {
                                id: sc::AssetId(descriptor),
                                revision: sc::Revision([0x41; 32]),
                            },
                            first: transfer(first),
                            second: transfer(second),
                        },
                    },
                }
            },
        )
        .collect::<Vec<_>>()
        .into();
    ContentDescription {
        gameplay,
        revision,
        allocations,
        roles,
        requests,
    }
}

// Movies cannot author the complete destination controls of a sealed portable request.
fn destination(
    word: u16,
    step: u16,
    lanes: ByteLanes,
    remap: u8,
    increment_on_high: bool,
) -> TileDestination {
    TileDestination {
        word,
        step,
        lanes,
        remap,
        increment_on_high,
    }
}

// Movies cannot supply these twelve role registrations, allocation IDs and complete tails.
pub(crate) fn supplied() -> ContentDescription {
    description(
        [
            (BodyTop, Loaded, 0x81, 0, Little),
            (BodyBottom, Loaded, 0x82, 1, Big),
            (PositionFallback, Native, 0x83, 2, Little),
            (Landing, Native, 0x84, 1, Big),
            (Crouching, Native, 0x85, 0, Little),
            (Drained, Native, 0x86, 2, Big),
            (TilePose, Loaded, 0x87, 0, Little),
            (TileDefinition, Native, 0x88, 1, Big),
            (TileTop, Loaded, 0x89, 0, Big),
            (TileBottom, Loaded, 0x8a, 2, Little),
            (UploadDescriptor, Native, 0x8b, 1, Little),
            (UploadDescriptor, Loaded, 0x8c, 0, Big),
        ],
        [
            (
                (Top, 0x9101, 1, 203),
                (0, 201, [0x21, 0x00], 32768, 0),
                (0, 202, [0x23, 0x00], 16, 32),
                destination(0x7000, 1, ByteLanes::Both, 0, true),
            ),
            (
                (Bottom, 0x9203, 1, 203),
                (2, 202, [0x23, 0x00], 16, 1),
                (0, 201, [0x21, 0x00], 32768, 65536),
                destination(0x7100, 1, ByteLanes::Both, 0, true),
            ),
        ],
        [0x71; 32],
        sc::Revision([0x61; 32]),
        8,
        [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x7a, 0x8b],
    )
}

// Movies cannot independently author the expected complete content and request data.
pub(crate) fn expected() -> ContentDescription {
    description(
        [
            (BodyTop, Loaded, 0x81, 0, Little),
            (BodyBottom, Loaded, 0x82, 1, Big),
            (PositionFallback, Native, 0x83, 2, Little),
            (Landing, Native, 0x84, 1, Big),
            (Crouching, Native, 0x85, 0, Little),
            (Drained, Native, 0x86, 2, Big),
            (TilePose, Loaded, 0x87, 0, Little),
            (TileDefinition, Native, 0x88, 1, Big),
            (TileTop, Loaded, 0x89, 0, Big),
            (TileBottom, Loaded, 0x8a, 2, Little),
            (UploadDescriptor, Native, 0x8b, 1, Little),
            (UploadDescriptor, Loaded, 0x8c, 0, Big),
        ],
        [
            (
                (Top, 0x9101, 1, 203),
                (0, 201, [0x21, 0x00], 32768, 0),
                (0, 202, [0x23, 0x00], 16, 32),
                destination(0x7000, 1, ByteLanes::Both, 0, true),
            ),
            (
                (Bottom, 0x9203, 1, 203),
                (2, 202, [0x23, 0x00], 16, 1),
                (0, 201, [0x21, 0x00], 32768, 65536),
                destination(0x7100, 1, ByteLanes::Both, 0, true),
            ),
        ],
        [0x71; 32],
        sc::Revision([0x61; 32]),
        8,
        [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x7a, 0x8b],
    )
}

// Movies cannot author whole request payloads or zero-count admission at a source boundary.
fn admit(
    supplied: ContentDescription,
    expected: &ContentDescription,
) -> (Result<ValidatedContent, DisplayError>, usize) {
    let reference = |id| sc::AssetRef {
        id: sc::AssetId(id),
        revision: sc::Revision([0x41; 32]),
    };
    let media = sc::SceneMedia::new(
        sc::Revision([0x41; 32]),
        vec![
            sc::SceneAsset::Atlas(sc::TileAtlas {
                reference: reference(201),
                encoding: sc::TileEncoding::Affine8,
                tile_count: 512,
                data: vec![0x21; 32768].into(),
            }),
            sc::SceneAsset::Atlas(sc::TileAtlas {
                reference: reference(202),
                encoding: sc::TileEncoding::Planar4,
                tile_count: 1,
                data: vec![0x23; 16].into(),
            }),
            sc::SceneAsset::Room(reference(203)),
            sc::SceneAsset::Room(reference(204)),
        ],
        vec![],
        vec![],
    )
    .unwrap();
    let mut checked = 0;
    let result = ValidatedContent::construct(supplied, expected, |request| {
        let pointer = [0x9101, 0x9203][checked];
        checked += 1;
        request.validate(pointer, &media)
    });
    (result, checked)
}

// Movies cannot alter unused allocation tails, role bindings or one registration key.
pub fn samus_content_rejects_allocation_and_role_changes() {
    let (supplied, expected) = (supplied(), expected());
    assert!(!Arc::ptr_eq(
        &supplied.allocations[&AllocationId([0x81; 32])],
        &expected.allocations[&AllocationId([0x81; 32])]
    ));
    let (sealed, checked) = admit(supplied, &expected);
    assert_eq!(checked, 2);
    assert_eq!(sealed.unwrap().description().roles.len(), 12);
    for case in 0..6 {
        let mut supplied = self::supplied();
        match case {
            0 => {
                Arc::make_mut(
                    &mut Arc::make_mut(
                        supplied
                            .allocations
                            .get_mut(&AllocationId([0x81; 32]))
                            .unwrap(),
                    )
                    .bytes,
                )[7] = 0x8a
            }
            1 => {
                supplied
                    .roles
                    .get_mut(&(BodyTop, Loaded))
                    .unwrap()
                    .allocation = AllocationId([0x82; 32])
            }
            2 => {
                supplied
                    .roles
                    .get_mut(&(Landing, Native))
                    .unwrap()
                    .native_order = Little
            }
            3 => {
                supplied
                    .roles
                    .get_mut(&(PositionFallback, Native))
                    .unwrap()
                    .mapped_base = 3
            }
            4 => {
                supplied.roles.remove(&(UploadDescriptor, Loaded));
            }
            _ => {
                supplied.roles.remove(&(UploadDescriptor, Native));
            }
        }
        let (result, checked) = admit(supplied, &expected);
        assert_eq!(result.err(), Some(DisplayError::Incomplete));
        assert_eq!(checked, 0);
    }
}

// Movies cannot replace authored request halves, identities, destinations or inactive tails.
pub fn samus_content_rejects_authored_request_changes() {
    for case in 0..9 {
        let (mut supplied, expected) = (supplied(), expected());
        let binding = &mut Arc::make_mut(&mut supplied.requests)[0];
        let TransferSource::Asset(source) = &mut binding.request.half.second.source else {
            unreachable!()
        };
        match case {
            0 => binding.half = Bottom,
            1 => binding.request.pointer = 0x9103,
            2 => binding.request.half.enabled = 2,
            3 => binding.request.half.descriptor.id = sc::AssetId(204),
            4 => binding.request.half.first.destination.word = 0x7001,
            5 => binding.request.half.first.count = 2,
            6 => {
                *source = SourceSpan {
                    asset: sc::AssetRef {
                        id: sc::AssetId(201),
                        revision: sc::Revision([0x41; 32]),
                    },
                    bytes: [0x21, 0x00].repeat(32768).into(),
                    offset: 65536,
                }
            }
            7 => source.offset = 31,
            _ => binding.request.half.second.count = 2,
        }
        let (result, checked) = admit(supplied, &expected);
        assert_eq!(result.err(), Some(DisplayError::Incomplete));
        assert_eq!(checked, 0);
    }
}

// Movies cannot give matching declarations with malformed full payloads or a first-zero shortfall.
pub fn samus_content_rejects_invalid_payload_and_transfer_extents() {
    for case in 0..3 {
        let (mut supplied, mut expected) = (supplied(), expected());
        for description in [&mut supplied, &mut expected] {
            let half = &mut Arc::make_mut(&mut description.requests)[0].request.half;
            let transfer = if case == 1 {
                &mut half.second
            } else {
                &mut half.first
            };
            let TransferSource::Asset(source) = &mut transfer.source else {
                unreachable!()
            };
            if case == 2 {
                source.offset = 1;
            } else {
                let last = source.bytes.len() - 1;
                Arc::make_mut(&mut source.bytes)[last] = 0x7e;
            }
        }
        let (result, checked) = admit(supplied, &expected);
        assert_eq!(result.err(), Some(DisplayError::Source));
        assert_eq!(checked, 1);
    }
}

// Movies cannot substitute a caller's same-identity declaration into a prepared sealed lease.
pub fn samus_content_prepared_lease_retains_sealed_allocation() {
    let (supplied, expected) = (supplied(), expected());
    let identity = AllocationId([0x84; 32]);
    let mut external = expected.allocations[&identity].clone();
    let (sealed, checked) = admit(supplied, &expected);
    let sealed = sealed.unwrap();
    assert_eq!(checked, 2);
    let bytes = Arc::make_mut(&mut Arc::make_mut(&mut external).bytes);
    bytes[1] = 0xee;
    bytes[7] = 0xef;
    assert_eq!(
        (external.identity, external.revision, external.extent),
        (identity, sc::Revision([0x61; 32]), 8)
    );
    assert_eq!(
        external.bytes.as_ref(),
        &[0x11, 0xee, 0x33, 0x44, 0x55, 0x66, 0x7a, 0xef]
    );
    let lease = sealed.acquire_prepared(Landing, Native, 0).unwrap();
    assert_eq!(lease.identity(), (Landing, Native, identity));
    assert_eq!(lease.native_word(0), Ok(0x2233));
    assert_eq!(lease.little_word(0), Ok(0x3322));
    assert_eq!(lease.byte(6), Ok(0x8b));
    assert_eq!(
        sealed.description().allocations[&identity].bytes.as_ref(),
        &[0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x7a, 0x8b]
    );
}
