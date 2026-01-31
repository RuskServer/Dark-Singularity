// src/lib.rs
use crate::core::singularity::Singularity;
use jni::JNIEnv;
use jni::objects::{JClass, JFloatArray, JIntArray, JString};
use jni::sys::{jfloat, jfloatArray, jint, jlong, jsize,jintArray};

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
    inputs: JFloatArray,
) -> jint {
    let singularity = unsafe { &mut *(handle as *mut Singularity) };

    let input_vec: Vec<f32> = {
        let len = env.get_array_length(&inputs).unwrap_or(0) as usize;
        let mut buf = vec![0.0f32; len];
        env.get_float_array_region(&inputs, 0, &mut buf).unwrap_or(());
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
    inputs: JFloatArray,
) -> jintArray {
    let singularity = unsafe { &mut *(handle as *mut Singularity) };
    
    let len = env.get_array_length(&inputs).unwrap_or(0) as usize;
    let mut buf = vec![0.0f32; len];
    env.get_float_array_region(&inputs, 0, &mut buf).unwrap_or(());
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
    reward: jfloat,
) {
    let singularity = unsafe { &mut *(handle as *mut Singularity) };
    // 最後に選択されたアクション群に対して報酬を適用
    singularity.learn(reward as f32);
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
) -> jfloat {
    let singularity = unsafe { &*(handle as *const Singularity) };
    singularity.system_temperature as jfloat
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_lunar_1prototype_dark_1singularity_1api_Singularity_getGliaActivityNative(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jfloat {
    let singularity = unsafe { &*(handle as *const Singularity) };
    // Horizon のバッファ状況から介入レベル(0.0-1.0)を取得
    singularity.horizon.get_intervention_level() as jfloat
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_lunar_1prototype_dark_1singularity_1api_Singularity_getActionScoreNative(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    action_idx: jint,
) -> jfloat {
    let singularity = unsafe { &*(handle as *const Singularity) };

    // 現在の最新状態(last_state_idx)における、指定アクションのQ値を取得
    let state_offset = singularity.last_state_idx * 8;
    let idx = state_offset + (action_idx as usize);

    if idx < singularity.q_table.len() {
        // 疲労度(fatigue_map)による減衰も考慮した「生のスコア」を返す
        let q_value = singularity.q_table[idx];
        let fatigue = singularity.fatigue_map[action_idx as usize];
        (q_value - (fatigue * 2.0)) as jfloat
    } else {
        0.0f32
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_lunar_1prototype_dark_1singularity_1api_Singularity_getFrustration(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jfloat {
    let singularity = unsafe { &*(handle as *const Singularity) };
    singularity.frustration as jfloat
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_lunar_1prototype_dark_1singularity_1api_Singularity_getAdrenaline(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jfloat {
    let singularity = unsafe { &*(handle as *const Singularity) };
    singularity.adrenaline as jfloat
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_lunar_1prototype_dark_1singularity_1api_Singularity_getNeuronStates(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jfloatArray {
    let singularity = unsafe { &*(handle as *const Singularity) };
    let states: Vec<f32> = singularity.nodes.iter().map(|n| n.state).collect();

    // 1. ラッパーオブジェクト(JFloatArray)を作成
    let output = env.new_float_array(states.len() as jsize).unwrap();

    // 2. 値をセット
    env.set_float_array_region(&output, 0, &states).unwrap();

    // 3. 重要：.into_raw() を呼び出して jfloatArray (ポインタ) に変換して返す
    output.into_raw()
}

// --- New Features: Save/Load ---

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