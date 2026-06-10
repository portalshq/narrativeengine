use narrativeengine_core::{
    create_block_json, generate_candidate_json, render_lore_summary_json, LabConfig,
    NarrativeBlock, NarrativeLore,
};

#[test]
fn json_contract_is_stable() {
    let block_json = create_block_json("intro", "A signal appears in the archive.").unwrap();
    assert_eq!(
        block_json,
        r#"{"id":"intro","content":"A signal appears in the archive."}"#
    );

    let block: NarrativeBlock = serde_json::from_str(&block_json).unwrap();
    let lore = NarrativeLore {
        id: "lore-1".to_string(),
        title: "Archive Signal".to_string(),
        blocks: vec![block],
    };
    let config = LabConfig::default();

    let candidate_json = generate_candidate_json(
        &serde_json::to_string(&lore).unwrap(),
        &serde_json::to_string(&config).unwrap(),
    )
    .unwrap();

    assert!(candidate_json.contains(r#""id":"candidate-lore-1-7""#));
    assert_eq!(
        render_lore_summary_json(&serde_json::to_string(&lore).unwrap()).unwrap(),
        "Archive Signal contains 1 block(s) and 6 word(s)."
    );
}
