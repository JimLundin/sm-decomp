//! Focused source-fixture evidence, excluding production timing and indexed acceptance.

use super::*;
use game::production::frame::borrowed_gameplay::{
    GameplayDispatchReceipt, GameplayRoute, SceneController,
};
use game::production::frame::borrowed_player_preparation::{
    PlayerEntryCompletions, PlayerEntryCompletionsPin, PlayerEntryOffers, PlayerEntryOffersPin,
    PlayerEntryOrigins, PlayerEntryOriginsPin, PlayerPreparationFault,
};

struct Fixture {
    bytes: Vec<u8>,
    entry: String,
    labels: String,
    links: String,
    rom: Rom,
    content: MovementContent,
}

impl Fixture {
    fn load() -> Self {
        let packet = std::path::PathBuf::from(std::env::var_os("SM_PLAYER_ENTRY_PACKET").unwrap());
        let rom_path = std::path::PathBuf::from(std::env::var_os("SM_PLAYER_ROM").unwrap());
        Self {
            bytes: std::fs::read(std::env::var_os("SM_FRAME_ORIGIN_WRAM").unwrap()).unwrap(),
            entry: std::fs::read_to_string(packet.join("ENTRY-MOTIONRESET-CURRENT.private.toml"))
                .unwrap(),
            labels: std::fs::read_to_string(packet.join("ENTRY-SEMANTIC-LABELS.private.toml"))
                .unwrap(),
            links: std::fs::read_to_string(packet.join("ENTRY-CONTENT-EFFECT-LINKS.private.toml"))
                .unwrap(),
            rom: Rom::new(std::fs::read(&rom_path).unwrap()).unwrap(),
            content: crate::replay::movement::Content::load(&rom_path)
                .unwrap()
                .movement,
        }
    }

    fn pose(&self) -> DeferredPoseInputReceipt {
        pose_offer(
            &self.bytes,
            &self.entry,
            &self.labels,
            &self.rom,
            &self.content,
        )
        .unwrap()
    }

    fn suit(&self) -> DeferredSuitPaletteReceipt {
        suit_offer(&self.bytes, &self.entry, &self.links, &self.rom).unwrap()
    }

    fn environment(&self) -> DeferredEnvironmentReceipt {
        environment_offer(
            &self.bytes,
            &self.entry,
            &self.labels,
            &self.links,
            &self.rom,
        )
        .unwrap()
    }

    fn acceleration(&self) -> types::Fixed {
        let links = EffectLinks::parse(&self.links).unwrap();
        let read = |name| {
            let link = links
                .effect_link
                .iter()
                .find(|link| link.owner == "Environment" && link.terminal_field == name)
                .unwrap();
            let bytes = self
                .rom
                .lorom()
                .read(
                    RomAddress::new(link.rom_bank.unwrap(), link.rom_offset.unwrap()),
                    usize::from(link.byte_width),
                )
                .unwrap();
            u16::try_from(scalar(bytes, 0, link.byte_width).unwrap()).unwrap()
        };
        types::fixed(
            core::num::Wrapping(read("VerticalAccelerationWhole")),
            read("VerticalAccelerationFraction"),
        )
    }
}

fn route() -> GameplayDispatchReceipt {
    GameplayDispatchReceipt {
        scene: SceneController::MainGameplay,
        route: GameplayRoute::MainGameplay,
    }
}

fn changed_member(text: &str, collection: &str, field_name: &str, key: &str) -> String {
    let mut parsed: toml::Value = toml::from_str(text).unwrap();
    let member = parsed
        .get_mut(collection)
        .unwrap()
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|member| {
            member
                .get("name")
                .or_else(|| member.get("terminal_field"))
                .and_then(toml::Value::as_str)
                == Some(field_name)
        })
        .unwrap();
    let value = member.get_mut(key).unwrap();
    *value = toml::Value::Integer(value.as_integer().unwrap() ^ 1);
    toml::to_string(&parsed).unwrap()
}

#[test]
fn player_entry_origin_offer_and_completion_shapes_remain_pinned() {
    assert!(canonical::model::pin_holds::<PlayerEntryOriginsPin>().is_ok());
    assert!(canonical::model::pin_holds::<PlayerEntryOffersPin>().is_ok());
    assert!(canonical::model::pin_holds::<PlayerEntryCompletionsPin>().is_ok());
}

#[test]
#[ignore = "requires accepted entry/current/labels/content links, complete origin and checked ROM"]
fn first_three_receipts_reopen_current_offers_and_completions() {
    use canonical::model::Model;

    let fixture = Fixture::load();
    let origins = Model::<PlayerEntryOriginsPin>(Box::new(PlayerEntryOrigins {
        pose_input: current_pose(&fixture.bytes, &fixture.entry).unwrap(),
        suit_palette: current_suit(&fixture.bytes, &fixture.entry).unwrap(),
        environment: current_environment(&fixture.bytes, &fixture.entry, &fixture.labels).unwrap(),
    }));
    let restored_origins: Model<PlayerEntryOriginsPin> =
        canonical::from_bytes(&canonical::to_bytes(&origins)).unwrap();
    assert!(restored_origins.eq(&origins));
    let offers = Model::<PlayerEntryOffersPin>(Box::new(PlayerEntryOffers {
        suit_palette: fixture.suit(),
        environment: fixture.environment(),
    }));
    let restored_offers: Model<PlayerEntryOffersPin> =
        canonical::from_bytes(&canonical::to_bytes(&offers)).unwrap();
    assert!(restored_offers.eq(&offers));
    let origins = restored_origins.0;
    let offers = restored_offers.0;
    let completions = Model::<PlayerEntryCompletionsPin>(Box::new(PlayerEntryCompletions {
        pose_input: Some(
            fixture
                .pose()
                .complete(&origins.pose_input, &fixture.content, &route())
                .unwrap(),
        ),
        suit_palette: Some(
            offers
                .suit_palette
                .complete(&origins.suit_palette, &route())
                .unwrap(),
        ),
        environment: Some(
            offers
                .environment
                .complete(&origins.environment, fixture.acceleration(), &route())
                .unwrap(),
        ),
    }));
    let restored: Model<PlayerEntryCompletionsPin> =
        canonical::from_bytes(&canonical::to_bytes(&completions)).unwrap();
    assert!(restored.eq(&completions));
}

#[test]
#[ignore = "requires accepted entry/current/labels/content links, complete origin and checked ROM"]
fn first_three_receipts_consume_actual_current_and_content() {
    let fixture = Fixture::load();
    let pose = fixture.pose();
    let suit = fixture.suit();
    let environment = fixture.environment();
    let pose_current = current_pose(&fixture.bytes, &fixture.entry).unwrap();
    let suit_current = current_suit(&fixture.bytes, &fixture.entry).unwrap();
    let environment_current =
        current_environment(&fixture.bytes, &fixture.entry, &fixture.labels).unwrap();
    assert!(pose.dependency.eq(&pose_current));
    assert!(suit.dependency.eq(&suit_current));
    assert!(environment.dependency.eq(&environment_current));
    assert!(
        pose.clone()
            .complete(&pose_current, &fixture.content, &route())
            .unwrap()
            .terminal
            .eq(&pose.terminal)
    );
    assert!(
        suit.clone()
            .complete(&suit_current, &route())
            .unwrap()
            .terminal
            .eq(&suit.terminal)
    );
    assert!(
        environment
            .clone()
            .complete(&environment_current, fixture.acceleration(), &route())
            .unwrap()
            .terminal
            .eq(&environment.terminal)
    );
    let mut changed_pose = pose_current.clone();
    changed_pose.held.0 ^= 1;
    assert!(
        pose.clone()
            .complete(&changed_pose, &fixture.content, &route())
            .is_err_and(|fault| fault == PlayerPreparationFault::Dependency)
    );
    let mut changed_suit = suit_current;
    changed_suit.equipment[types::Item::GravitySuit] =
        !changed_suit.equipment[types::Item::GravitySuit];
    assert!(
        suit.complete(&changed_suit, &route())
            .is_err_and(|fault| fault == PlayerPreparationFault::Dependency)
    );
    let mut changed_environment = environment_current.clone();
    changed_environment.centre_y ^= 1;
    assert!(
        environment
            .clone()
            .complete(&changed_environment, fixture.acceleration(), &route())
            .is_err_and(|fault| fault == PlayerPreparationFault::Dependency)
    );
    assert!(
        environment
            .complete(
                &environment_current,
                fixture.acceleration() + core::num::Wrapping(1),
                &route()
            )
            .is_err_and(|fault| fault == PlayerPreparationFault::Content)
    );
    let mut unsupported_route = route();
    unsupported_route.scene = SceneController::Other;
    assert!(
        pose.complete(&pose_current, &fixture.content, &unsupported_route)
            .is_err_and(|fault| fault == PlayerPreparationFault::Unsupported)
    );
}

#[test]
#[ignore = "requires accepted entry/current/labels/content links, complete origin and checked ROM"]
fn every_current_field_and_content_effect_link_is_required() {
    let fixture = Fixture::load();
    let source = Projection::parse(&fixture.entry).unwrap();
    for field in source
        .current_field
        .iter()
        .filter(|field| field.owner != "MotionReset")
    {
        let mut changed = fixture.bytes.clone();
        *changed.get_mut(field.private_wram_offset).unwrap() ^= 1;
        let fault = match field.owner.as_str() {
            "PoseInput" => current_pose(&changed, &fixture.entry).err(),
            "SuitPalette" => current_suit(&changed, &fixture.entry).err(),
            "Environment" => current_environment(&changed, &fixture.entry, &fixture.labels).err(),
            _ => panic!("unexpected current owner"),
        };
        assert!(fault.eq(&Some(PlayerReceiptFault::Dependency)));
    }
    for length in [0, 131_071, 131_073] {
        assert!(
            current_pose(&vec![0; length], &fixture.entry)
                .is_err_and(|fault| fault == PlayerReceiptFault::Length)
        );
    }
}

#[test]
#[ignore = "requires qualified native entry and checked content links"]
fn borrowed_effects_require_their_qualified_content() {
    let fixture = Fixture::load();
    let wrong_radius = changed_member(
        &fixture.entry,
        "borrowed_terminal_field",
        "SAMUS_Y_RADIUS",
        "qualified_borrowed_terminal_encoding",
    );
    assert!(
        pose_offer(
            &fixture.bytes,
            &wrong_radius,
            &fixture.labels,
            &fixture.rom,
            &fixture.content
        )
        .is_err_and(|fault| fault == PlayerReceiptFault::Content)
    );
    let wrong_acceleration = changed_member(
        &fixture.entry,
        "borrowed_terminal_field",
        "VerticalAccelerationFraction",
        "qualified_borrowed_terminal_encoding",
    );
    assert!(
        environment_offer(
            &fixture.bytes,
            &wrong_acceleration,
            &fixture.labels,
            &fixture.links,
            &fixture.rom
        )
        .is_err_and(|fault| fault == PlayerReceiptFault::Content)
    );
    let wrong_suit = changed_member(
        &fixture.entry,
        "borrowed_terminal_field",
        "SuitPaletteDisposition",
        "qualified_borrowed_terminal_encoding",
    );
    assert!(
        suit_offer(&fixture.bytes, &wrong_suit, &fixture.links, &fixture.rom)
            .is_err_and(|fault| fault == PlayerReceiptFault::Content)
    );
    let mut swapped: toml::Value = toml::from_str(&fixture.links).unwrap();
    for link in swapped
        .get_mut("effect_link")
        .unwrap()
        .as_array_mut()
        .unwrap()
    {
        if link.get("owner").and_then(toml::Value::as_str) == Some("Environment") {
            let name = link.get_mut("imported_operand_name").unwrap();
            *name = toml::Value::String(if name.as_str().unwrap().ends_with("_1") {
                "EnvironmentSurfaceOrAccelerationContent_2".to_owned()
            } else {
                "EnvironmentSurfaceOrAccelerationContent_1".to_owned()
            });
        }
    }
    assert!(
        environment_offer(
            &fixture.bytes,
            &fixture.entry,
            &fixture.labels,
            &toml::to_string(&swapped).unwrap(),
            &fixture.rom
        )
        .is_err_and(|fault| fault == PlayerReceiptFault::Content)
    );
    let wrong_content = changed_member(
        &fixture.entry,
        "imported_content",
        "PoseRadiusOrTransitionRequirement_0",
        "declared_encoding",
    );
    assert!(
        pose_offer(
            &fixture.bytes,
            &wrong_content,
            &fixture.labels,
            &fixture.rom,
            &fixture.content
        )
        .is_err_and(|fault| fault == PlayerReceiptFault::Content)
    );
}

#[test]
#[ignore = "requires accepted remaining-five packet, complete current origin and checked ROM"]
fn remaining_five_import_without_future_selected_owners_and_reject_current_corruption() {
    let bytes = std::fs::read(std::env::var_os("SM_FRAME_ORIGIN_WRAM").unwrap()).unwrap();
    let projection =
        std::fs::read_to_string(std::env::var_os("SM_PLAYER_CONTINUATION_PACKET").unwrap())
            .unwrap();
    let rom = Rom::new(std::fs::read(std::env::var_os("SM_PLAYER_ROM").unwrap()).unwrap()).unwrap();
    let (origins, offers) = player_continuation(&bytes, &projection, &rom).unwrap();
    assert!(origins.actors.eq(&offers.actors.dependency));
    assert!(
        offers
            .actors
            .required
            .interactive_order
            .as_slice()
            .eq(&offers.actors.required.active_order[1..])
    );
    let source = BorrowedProjection::parse(&projection).unwrap();
    for field in &source.current_field {
        let mut changed = bytes.clone();
        *changed.get_mut(field.field.private_wram_offset).unwrap() ^= 1;
        assert!(
            player_continuation(&changed, &projection, &rom)
                .is_err_and(|fault| fault == PlayerReceiptFault::Dependency)
        );
    }
    for name in [
        "ActorInstructionTimerFirst",
        "ActorInstructionTimerSecond",
        "ActorGraphicsFlagsFirst",
        "ActorGraphicsFlagsSecond",
    ] {
        let mut wrong: toml::Value = toml::from_str(&projection).unwrap();
        let terminal = wrong
            .get_mut("borrowed_terminal_field")
            .unwrap()
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|terminal| terminal.get("name").and_then(toml::Value::as_str) == Some(name))
            .unwrap();
        terminal["terminal_relation"] = toml::Value::String("UnqualifiedRelation".to_owned());
        assert!(
            player_continuation(&bytes, &toml::to_string(&wrong).unwrap(), &rom)
                .is_err_and(|fault| fault == PlayerReceiptFault::Unsupported)
        );
    }
    // Retained counters are data, so changing them remains admissible and changes the origin.
    for counter in &source.current_retained_quantity {
        let mut changed = bytes.clone();
        *changed.get_mut(counter.private_wram_offset).unwrap() ^= 1;
        let (changed_origins, changed_offers) =
            player_continuation(&changed, &projection, &rom).unwrap();
        assert!(!changed_origins.eq(&origins));
        assert!(changed_offers.actors.required.eq(&offers.actors.required));
    }
    for field in &source.actor_geometry {
        let mut changed = bytes.clone();
        *changed.get_mut(field.private_wram_offset).unwrap() ^= 1;
        assert!(
            player_continuation(&changed, &projection, &rom)
                .is_err_and(|fault| fault == PlayerReceiptFault::Dependency)
        );
    }
}
