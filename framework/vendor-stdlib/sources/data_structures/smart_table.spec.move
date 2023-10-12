spec diem_std::smart_table {
    spec SmartTable {
        pragma intrinsic = map,
            map_new = new,
            map_destroy_empty = destroy_empty,
            map_len = length,
            map_has_key = contains,
            map_add_no_override = add,
            map_add_override_if_exists = upsert,
            map_del_must_exist = remove,
            map_borrow = borrow,
            map_borrow_mut = borrow_mut,
            map_borrow_mut_with_default = borrow_mut_with_default,
            map_spec_get = spec_get,
            map_spec_set = spec_set,
            map_spec_del = spec_remove,
            map_spec_len = spec_len,
            map_spec_has_key = spec_contains;
    }

    spec new_with_config<K: copy + drop + store, V: store>(num_initial_buckets: u64, split_load_threshold: u8, target_bucket_size: u64): SmartTable<K, V> {
        pragma verify = false;
    }

    spec destroy<K: drop, V: drop>(table: SmartTable<K, V>) {
        pragma verify = false;
    }

    spec split_one_bucket<K, V>(table: &mut SmartTable<K, V>) {
        pragma verify= false;
    }

    spec bucket_index(level: u8, num_buckets: u64, hash: u64): u64 {
        pragma verify = false;
    }

    spec borrow_with_default<K: copy + drop, V>(table: &SmartTable<K, V>, key: K, default: &V): &V {
        pragma verify = false;
    }

    spec load_factor<K, V>(table: &SmartTable<K, V>): u64 {
        pragma verify = false;
    }

    spec update_split_load_threshold<K, V>(table: &mut SmartTable<K, V>, split_load_threshold: u8) {
        pragma verify = false;
    }

    spec update_target_bucket_size<K, V>(table: &mut SmartTable<K, V>, target_bucket_size: u64) {
        pragma verify = false;
    }

    // Specification functions for tables
    spec native fun spec_len<K, V>(t: SmartTable<K, V>): num;
    spec native fun spec_contains<K, V>(t: SmartTable<K, V>, k: K): bool;
    spec native fun spec_set<K, V>(t: SmartTable<K, V>, k: K, v: V): SmartTable<K, V>;
    spec native fun spec_remove<K, V>(t: SmartTable<K, V>, k: K): SmartTable<K, V>;
    spec native fun spec_get<K, V>(t: SmartTable<K, V>, k: K): V;
}
