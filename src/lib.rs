// src/lib.rs
use crate::core::singularity::Singularity;
use jni::JNIEnv;
use jni::objects::{JClass, JDoubleArray, JIntArray, JString};
use jni::sys::{jdouble, jdoubleArray, jint, jlong, jsize, jintArray};

pub mod core;

// インスタンスを生成して Java にポインタ(jlong)として返す
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_lunar_1prototype_dark_1singularity_1api_Singularity_initNativeSingularity(
    env: JNIEnv,
    _class: JClass,
    state_size: jint,
    category_sizes: JIntArray,
) -> jlong {
    // JNIのint配列をRustのVec<usize>に変換
    let len = env.get_array_length(&category_sizes).unwrap_or(0) as usize;
    let mut cat_buf = vec![0i32; len];
    env.get_int_array_region(&category_sizes, 0, &mut cat_buf).unwrap_or(());
    
    let cat_sizes: Vec<usize> = cat_buf.into_iter().map(|s| s as usize).collect();

    let singularity = Box::new(Singularity::new(state_size as usize, cat_sizes));
    Box::into_raw(singularity) as jlong
}

// Java からもらったポインタを使って計算する
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_lunar_1prototype_dark_1singularity_1api_Singularity_selectActionNative(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    inputs: JDoubleArray,
) -> jint {
    let singularity = unsafe { &mut *(handle as *mut Singularity) };

    let input_vec: Vec<f64> = {
        let len = env.get_array_length(&inputs).unwrap_or(0) as usize;
        let mut buf = vec![0.0f64; len];
        env.get_double_array_region(&inputs, 0, &mut buf).unwrap_or(());
        buf
    };

    let state_idx = if !input_vec.is_empty() { input_vec[0] as usize } else { 0 };

    // 最初のカテゴリーのベストアクションを返す (単一アクション互換)
    let actions = singularity.select_actions(state_idx);
    actions.first().cloned().unwrap_or(0) as jint
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_lunar_1prototype_dark_1singularity_1api_Singularity_selectActionsNative(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    inputs: JDoubleArray,
) -> jintArray {
    let singularity = unsafe { &mut *(handle as *mut Singularity) };
    
    let len = env.get_array_length(&inputs).unwrap_or(0) as usize;
    let mut buf = vec![0.0f64; len];
    env.get_double_array_region(&inputs, 0, &mut buf).unwrap_or(());
    let state_idx = if !buf.is_empty() { buf[0] as usize } else { 0 };

    let actions = singularity.select_actions(state_idx);

    let output = env.new_int_array(actions.len() as jsize).unwrap();
    env.set_int_array_region(&output, 0, &actions).unwrap();
    output.into_raw()
}

// 学習（経験の消化）を Rust 側で実行
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_lunar_1prototype_dark_1singularity_1api_Singularity_learnNative(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    reward: jdouble,
) {
    let singularity = unsafe { &mut *(handle as *mut Singularity) };
    // 最後に選択されたアクション群に対して報酬を適用
    singularity.learn(reward as f64);
}

// src/lib.rs

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_lunar_1prototype_dark_1singularity_1api_Singularity_destroyNativeSingularity(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    if handle != 0 {
        unsafe {
            // rawポインタをBoxに戻してスコープを抜けることで自動解放
            let _ = Box::from_raw(handle as *mut Singularity);
        }
        println!("DarkSingularity memory released.");
    }
}

// 他のパラメータをJava側に返す（Snapshot用）ゲッターも必要であればここに追加
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_lunar_1prototype_dark_1singularity_1api_Singularity_getSystemTemperature(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jdouble {
    let singularity = unsafe { &*(handle as *const Singularity) };
    singularity.system_temperature as jdouble
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_lunar_1prototype_dark_1singularity_1api_Singularity_getGliaActivityNative(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jdouble {
    let singularity = unsafe { &*(handle as *const Singularity) };
    // Horizon のバッファ状況から介入レベル(0.0-1.0)を取得
    singularity.horizon.get_intervention_level() as jdouble
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_lunar_1prototype_dark_1singularity_1api_Singularity_getActionScoreNative(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    action_idx: jint,
) -> jdouble {
    let singularity = unsafe { &mut *(handle as *mut Singularity) };

    // JNI 経由のスコア取得ではノイズを乗せない。複素版に戻す
    let mwso_scores = singularity.mwso.get_action_scores(0, singularity.action_size, 0.0, &[]);
    let idx = action_idx as usize;

    if idx < mwso_scores.len() {
        let wave_score = mwso_scores[idx];
        let fatigue = singularity.fatigue_map[idx];
        (wave_score - (fatigue * 2.0)) as jdouble
    } else {
        0.0f64
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_lunar_1prototype_dark_1singularity_1api_Singularity_getFrustration(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jdouble {
    let singularity = unsafe { &*(handle as *const Singularity) };
    singularity.frustration as jdouble
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_lunar_1prototype_dark_1singularity_1api_Singularity_getAdrenaline(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jdouble {
    let singularity = unsafe { &*(handle as *const Singularity) };
    singularity.adrenaline as jdouble
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_lunar_1prototype_dark_1singularity_1api_Singularity_setExplorationBetaNative(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    beta: jdouble,
) {
    let singularity = unsafe { &mut *(handle as *mut Singularity) };
    singularity.exploration_beta = beta as f64;
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_lunar_1prototype_dark_1singularity_1api_Singularity_getExplorationBetaNative(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jdouble {
    let singularity = unsafe { &*(handle as *const Singularity) };
    singularity.exploration_beta as jdouble
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_lunar_1prototype_dark_1singularity_1api_Singularity_setNeuronStateNative(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    idx: jint,
    state: jdouble,
) {
    let singularity = unsafe { &mut *(handle as *mut Singularity) };
    singularity.set_neuron_state(idx as usize, state as f64);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_lunar_1prototype_dark_1singularity_1api_Singularity_getNeuronStates(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jdoubleArray {
    let singularity = unsafe { &*(handle as *const Singularity) };
    let states: Vec<f64> = singularity.nodes.iter().map(|n| n.state).collect();

    // 1. ラッパーオブジェクト(JDoubleArray)を作成
    let output = env.new_double_array(states.len() as jsize).unwrap();

    // 2. 値をセット
    env.set_double_array_region(&output, 0, &states).unwrap();

    // 3. 重要：.into_raw() を呼び出して jdoubleArray (ポインタ) に変換して返す
    output.into_raw()
}

// --- New Features: Save/Load ---

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_lunar_1prototype_dark_1singularity_1api_Singularity_generateVisualSnapshotNative(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    path: JString,
) -> jint {
    let singularity = unsafe { &*(handle as *const Singularity) };
    let path_str: String = match env.get_string(&path) {
        Ok(s) => s.into(),
        Err(_) => return -1,
    };

    if singularity.generate_visual_snapshot(&path_str) { 0 } else { -1 }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_lunar_1prototype_dark_1singularity_1api_Singularity_saveNativeModel(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    path: JString,
) -> jint {
    let singularity = unsafe { &*(handle as *const Singularity) };

    // Java String -> Rust String
    let path_str: String = match env.get_string(&path) {
        Ok(s) => s.into(),
        Err(_) => return -1, // Error
    };

    match singularity.save_to_file(&path_str) {
        Ok(_) => 0, // Success
        Err(e) => {
            println!("Error saving model: {}", e);
            -2 // Save Error
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_lunar_1prototype_dark_1singularity_1api_Singularity_loadNativeModel(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    path: JString,
) -> jint {
    let singularity = unsafe { &mut *(handle as *mut Singularity) };

    // Java String -> Rust String
    let path_str: String = match env.get_string(&path) {
        Ok(s) => s.into(),
        Err(_) => return -1, // Error
    };

    match singularity.load_from_file(&path_str) {
        Ok(_) => 0, // Success
        Err(e) => {
            println!("Error loading model: {}", e);
            -2 // Load Error
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_lunar_1prototype_dark_1singularity_1api_Singularity_setActiveConditionsNative(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    condition_ids: JIntArray,
) {
    let singularity = unsafe { &mut *(handle as *mut Singularity) };
    let len = env.get_array_length(&condition_ids).unwrap_or(0) as usize;
    let mut buf = vec![0i32; len];
    env.get_int_array_region(&condition_ids, 0, &mut buf).unwrap_or(());
    
    singularity.set_active_conditions(&buf);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_lunar_1prototype_dark_1singularity_1api_Singularity_bootstrapNative(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    condition_indices: JIntArray,
    action_indices: JIntArray,
    strengths: JDoubleArray,
) {
    let singularity = unsafe { &mut *(handle as *mut Singularity) };
    
    let len = env.get_array_length(&condition_indices).unwrap_or(0) as usize;
    let mut conds = vec![0i32; len];
    let mut actions = vec![0i32; len];
    let mut str_vals = vec![0.0f64; len];

    env.get_int_array_region(&condition_indices, 0, &mut conds).unwrap_or(());
    env.get_int_array_region(&action_indices, 0, &mut actions).unwrap_or(());
    env.get_double_array_region(&strengths, 0, &mut str_vals).unwrap_or(());

    for i in 0..len {
        singularity.bootstrapper.add_hamiltonian_rule(conds[i], actions[i] as usize, str_vals[i]);
    }
}
