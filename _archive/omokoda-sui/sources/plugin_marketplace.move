/// Garden Marketplace — decentralized plugin registry with IPFS content addressing.
///
/// Publishers register plugin versions by anchoring their IPFS CID and a blake3
/// content hash on-chain. Clients verify the CID + hash before installing.
/// Publishers can yank a version to signal it should not be installed.
///
/// Ports Claw-code's officialMarketplace pattern to a trustless on-chain model:
/// instead of a GCS bucket, content lives on IPFS/Arweave; the Sui object graph
/// is the authoritative registry index.
module omokoda_sui::plugin_marketplace {

    use sui::object::{Self, UID};
    use sui::tx_context::{Self, TxContext};
    use sui::transfer;
    use sui::event;
    use std::string::String;

    // ── Error codes ──────────────────────────────────────────────────────────

    const E_NOT_PUBLISHER: u64 = 1;
    const E_ALREADY_YANKED: u64 = 2;
    const E_EMPTY_CID: u64 = 3;

    // ── Core objects ─────────────────────────────────────────────────────────

    /// A single published plugin version.
    /// Shared so anyone can read it; only the publisher can mutate it.
    struct GardenPlugin has key, store {
        id: UID,
        /// Human-readable plugin name (e.g. "omokoda-web-search")
        name: String,
        /// Semver string (e.g. "1.2.3")
        version: String,
        /// IPFS CIDv1 (or Arweave tx ID) of the plugin bundle — UTF-8 bytes
        content_cid: vector<u8>,
        /// blake3 hash of the bundle for client-side integrity verification
        content_hash: vector<u8>,
        /// Sui address of the publisher
        publisher: address,
        /// Epoch-millisecond timestamp of initial publication
        published_at: u64,
        /// Whether this version has been yanked (should not be installed)
        yanked: bool,
        /// Number of on-chain install acknowledgements (informational)
        install_count: u64,
    }

    // ── Events ───────────────────────────────────────────────────────────────
    // Note: event structs must have `copy + drop`; vectors lack `copy` so
    // content_cid is omitted — clients read it directly from the shared object.

    struct PluginPublished has copy, drop {
        plugin_id: address,
        name: String,
        version: String,
        publisher: address,
    }

    struct PluginYanked has copy, drop {
        plugin_id: address,
        name: String,
        version: String,
    }

    struct InstallAcknowledged has copy, drop {
        plugin_id: address,
        installer: address,
    }

    // ── Entry functions ───────────────────────────────────────────────────────

    /// Publish a new plugin version.
    /// The resulting GardenPlugin object is shared so anyone can read and verify it.
    public entry fun publish(
        name: String,
        version: String,
        content_cid: vector<u8>,
        content_hash: vector<u8>,
        ctx: &mut TxContext,
    ) {
        assert!(std::vector::length(&content_cid) > 0, E_EMPTY_CID);

        let plugin_uid = object::new(ctx);
        let plugin_addr = object::uid_to_address(&plugin_uid);
        let publisher = tx_context::sender(ctx);
        let published_at = tx_context::epoch_timestamp_ms(ctx);

        // Capture name/version strings for the event before moving into struct
        // (String has `copy` ability in Sui Move)
        let ev_name = name;
        let ev_version = version;

        let plugin = GardenPlugin {
            id: plugin_uid,
            name: ev_name,
            version: ev_version,
            content_cid,
            content_hash,
            publisher,
            published_at,
            yanked: false,
            install_count: 0,
        };

        event::emit(PluginPublished {
            plugin_id: plugin_addr,
            name: plugin.name,
            version: plugin.version,
            publisher: plugin.publisher,
        });

        transfer::share_object(plugin);
    }

    /// Yank a plugin version — only the original publisher may call this.
    public entry fun yank(plugin: &mut GardenPlugin, ctx: &mut TxContext) {
        assert!(plugin.publisher == tx_context::sender(ctx), E_NOT_PUBLISHER);
        assert!(!plugin.yanked, E_ALREADY_YANKED);
        plugin.yanked = true;

        event::emit(PluginYanked {
            plugin_id: object::uid_to_address(&plugin.id),
            name: plugin.name,
            version: plugin.version,
        });
    }

    /// Record an install acknowledgement; anyone may call this to increment the counter.
    public entry fun acknowledge_install(plugin: &mut GardenPlugin, ctx: &mut TxContext) {
        plugin.install_count = plugin.install_count + 1;
        event::emit(InstallAcknowledged {
            plugin_id: object::uid_to_address(&plugin.id),
            installer: tx_context::sender(ctx),
        });
    }

    // ── Read-only helpers ─────────────────────────────────────────────────────

    /// Verify a plugin is not yanked and its blake3 content hash matches.
    public fun verify(plugin: &GardenPlugin, expected_hash: &vector<u8>): bool {
        !plugin.yanked && &plugin.content_hash == expected_hash
    }

    /// Return (name, version, publisher, yanked, install_count).
    public fun get_info(plugin: &GardenPlugin): (String, String, address, bool, u64) {
        (plugin.name, plugin.version, plugin.publisher, plugin.yanked, plugin.install_count)
    }

    /// Return the IPFS CID bytes for the plugin bundle.
    public fun content_cid(plugin: &GardenPlugin): &vector<u8> {
        &plugin.content_cid
    }
}
