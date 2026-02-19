# rust-xgboost

Rust bindings for the [XGBoost](https://xgboost.ai) gradient boosting library.

Statically links XGBoost 3.0.5, built from source via cmake — no system-level XGBoost installation required.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
xgboost = { git = "https://github.com/o-love/rust-xgboost.git" }

[patch.crates-io]
xgboost-sys = { git = "https://github.com/o-love/rust-xgboost.git" }

```

Basic example:

```rust
use xgboost::{parameters, DMatrix, Booster};

fn main() {
    // training matrix with 5 training examples and 3 features
    let x_train = &[1.0, 1.0, 1.0,
                    1.0, 1.0, 0.0,
                    1.0, 1.0, 1.0,
                    0.0, 0.0, 0.0,
                    1.0, 1.0, 1.0];
    let num_rows = 5;
    let y_train = &[1.0, 1.0, 1.0, 0.0, 1.0];

    // convert training data into XGBoost's matrix format
    let mut dtrain = DMatrix::from_dense(x_train, num_rows).unwrap();
    dtrain.set_labels(y_train).unwrap();

    // test matrix with 1 row
    let x_test = &[0.7, 0.9, 0.6];
    let num_rows = 1;
    let y_test = &[1.0];
    let mut dtest = DMatrix::from_dense(x_test, num_rows).unwrap();
    dtest.set_labels(y_test).unwrap();

    // configure objectives, metrics, etc.
    let learning_params = parameters::learning::LearningTaskParametersBuilder::default()
        .objective(parameters::learning::Objective::BinaryLogistic)
        .build().unwrap();

    // configure the tree-based learning model's parameters
    let tree_params = parameters::tree::TreeBoosterParametersBuilder::default()
        .max_depth(2)
        .eta(1.0)
        .build().unwrap();

    // overall configuration for Booster
    let booster_params = parameters::BoosterParametersBuilder::default()
        .booster_type(parameters::BoosterType::Tree(tree_params))
        .learning_params(learning_params)
        .verbose(true)
        .build().unwrap();

    // specify datasets to evaluate against during training
    let evaluation_sets = &[(&dtrain, "train"), (&dtest, "test")];

    // overall configuration for training/evaluation
    let params = parameters::TrainingParametersBuilder::default()
        .dtrain(&dtrain)
        .boost_rounds(2)
        .booster_params(booster_params)
        .evaluation_sets(Some(evaluation_sets))
        .build().unwrap();

    // train model, and print evaluation data
    let bst = Booster::train(&params).unwrap();

    println!("{:?}", bst.predict(&dtest).unwrap());
}
```

See the [examples](examples/) directory for more detailed examples including custom objectives, GLMs, and multiclass classification.

## Features

### CUDA / GPU support

Enable GPU-accelerated training with the `cuda` feature flag:

```toml
[dependencies]
xgboost = { version = "0.3", features = ["cuda"] }
```

Requires a CUDA toolkit installation. A [Nix flake](flake.nix) is included for reproducible CUDA dev environments:

```sh
nix develop
```

### Feature name extraction

Retrieve feature names from a trained model:

```rust
let feature_names = bst.get_feature_names().unwrap();
```

### Model serialization

Save/load models in XGBoost's binary format or export the JSON config:

```rust
bst.save("model.bin").unwrap();
let bst = Booster::load("model.bin").unwrap();

let json_config = bst.save_json_config().unwrap();
```

## Platforms

Tested:
- Linux
- macOS

## License

MIT
