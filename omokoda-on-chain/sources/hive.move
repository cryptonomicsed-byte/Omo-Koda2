module omokoda::hive {
    use sui::object::{Self, UID};
    use sui::tx_context::{Self, TxContext};
    use sui::transfer;

    /// Hive node registration — Nautilus TEE nodes that provide compute.
    /// hive.move LAST: depends on stable Nautilus API.
    struct HiveNode has key, store {
        id: UID,
        node_id: vector<u8>,
        tee_attestation: vector<u8>,
        dopamine_capacity: u64,
        dopamine_allocated: u64,
        online: bool,
        registered_at: u64,
    }

    /// A compute allocation from the global Dopamine pool.
    struct DopamineAllocation has key, store {
        id: UID,
        agent_id: vector<u8>,
        node_id: vector<u8>,
        amount: u64,
        allocated_at: u64,
    }

    const DOPAMINE_TOTAL_POOL: u64 = 86_000_000_000;
    const E_CAPACITY_EXCEEDED: u64 = 1;
    const E_NODE_OFFLINE: u64 = 2;
    const E_INVALID_AMOUNT: u64 = 3;

    entry fun register_node(
        node_id: vector<u8>,
        tee_attestation: vector<u8>,
        dopamine_capacity: u64,
        registered_at: u64,
        ctx: &mut TxContext,
    ) {
        assert!(dopamine_capacity > 0, E_INVALID_AMOUNT);
        let node = HiveNode {
            id: object::new(ctx),
            node_id,
            tee_attestation,
            dopamine_capacity,
            dopamine_allocated: 0,
            online: true,
            registered_at,
        };
        transfer::transfer(node, tx_context::sender(ctx));
    }

    entry fun allocate(
        node: &mut HiveNode,
        agent_id: vector<u8>,
        amount: u64,
        allocated_at: u64,
        ctx: &mut TxContext,
    ) {
        assert!(node.online, E_NODE_OFFLINE);
        assert!(node.dopamine_allocated + amount <= node.dopamine_capacity, E_CAPACITY_EXCEEDED);
        node.dopamine_allocated = node.dopamine_allocated + amount;

        let alloc = DopamineAllocation {
            id: object::new(ctx),
            agent_id,
            node_id: node.node_id,
            amount,
            allocated_at,
        };
        transfer::transfer(alloc, tx_context::sender(ctx));
    }

    entry fun release(node: &mut HiveNode, amount: u64) {
        if (node.dopamine_allocated >= amount) {
            node.dopamine_allocated = node.dopamine_allocated - amount;
        } else {
            node.dopamine_allocated = 0;
        }
    }

    entry fun set_online(node: &mut HiveNode, online: bool) {
        node.online = online;
    }

    public fun available(node: &HiveNode): u64 {
        node.dopamine_capacity - node.dopamine_allocated
    }
    public fun dopamine_total_pool(): u64 { DOPAMINE_TOTAL_POOL }
    public fun is_online(node: &HiveNode): bool { node.online }
}
