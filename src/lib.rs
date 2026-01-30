// src/lib.rs
use crate::core::singularity::Singularity;
use jni::JNIEnv;
use jni::objects::{JClass, JFloatArray, JString};
use jni::sys::{jfloat, jfloatArray, jint, jlong, jsize};

pub mod core;

// インスタンスを生成して Java にポインタ(jlong)として返す
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_lunar_1prototype_deepwither_seeker_LiquidBrain_initNativeSingularity(
    _env: JNIEnv,
    _class: JClass,
) -> jlong {
    let singularity = Box::new(Singularity::new());
    Box::into_raw(singularity) as jlong
}

// Java からもらったポインタを使って計算する
// src/lib.rs の selectActionNative 部分

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_lunar_1prototype_deepwither_seeker_LiquidBrain_selectActionNative(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    inputs: JFloatArray,
) -> jint {
    let singularity = unsafe { &mut *(handle as *mut Singularity) };

    // --- 修正箇所：JNI配列からRustのVecへ ---
    // 1. 安全にJNI配列の要素を取得
    let input_vec: Vec<f32> = {
        // 配列を読み取ってコピーする（要素数を取得して変換）
        let len = env.get_array_length(&inputs).unwrap_or(0) as usize;
        let mut buf = vec![0.0f32; len];
        env.get_float_array_region(&inputs, 0, &mut buf)
            .unwrap_or(());
        buf
    };
    // ------------------------------------

    // input_vec を使って状態インデックスを計算するロジック（Java版のロジックに合わせて）
    // 一旦、Javaから渡される情報の先頭を state_idx と仮定するか、
    // あるいは Java 側で計算済みの idx を引数に追加するのもアリです。
    let state_idx = if !input_vec.is_empty() {
        input_vec[0] as usize
    } else {
        0
    };

    singularity.select_action(state_idx) as jint
}

// 学習（経験の消化）を Rust 側で実行
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_lunar_1prototype_deepwither_seeker_LiquidBrain_learnNative(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    reward: jfloat,
) {
    let singularity = unsafe { &mut *(handle as *mut Singularity) };
    singularity.learn(reward);
}

// src/lib.rs

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_lunar_1prototype_deepwither_seeker_LiquidBrain_destroyNativeSingularity(
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
pub extern "system" fn Java_com_lunar_1prototype_deepwither_seeker_LiquidBrain_getSystemTemperature(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jfloat {
    let singularity = unsafe { &*(handle as *const Singularity) };
    singularity.system_temperature as jfloat
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_lunar_1prototype_deepwither_seeker_LiquidBrain_getGliaActivityNative(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jfloat {
    let singularity = unsafe { &*(handle as *const Singularity) };
    // Horizon のバッファ状況から介入レベル(0.0-1.0)を取得
    singularity.horizon.get_intervention_level() as jfloat
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_lunar_1prototype_deepwither_seeker_LiquidBrain_getActionScoreNative(
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
pub extern "system" fn Java_com_lunar_1prototype_deepwither_seeker_LiquidBrain_getFrustration(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jfloat {
    let singularity = unsafe { &*(handle as *const Singularity) };
    singularity.frustration as jfloat
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_lunar_1prototype_deepwither_seeker_LiquidBrain_getAdrenaline(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jfloat {
    let singularity = unsafe { &*(handle as *const Singularity) };
    singularity.adrenaline as jfloat
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_lunar_1prototype_deepwither_seeker_LiquidBrain_getNeuronStates(
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
pub extern "system" fn Java_com_lunar_1prototype_deepwither_seeker_LiquidBrain_saveNativeModel(
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
pub extern "system" fn Java_com_lunar_1prototype_deepwither_seeker_LiquidBrain_loadNativeModel(
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
