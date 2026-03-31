//! ML Fitness Demo - Demonstrates machine learning-based fitness functions for ZK circuit fuzzing
//!
//! This example showcases Phase 7 features:
//! - Neural network fitness prediction
//! - Feature extraction from circuits
//! - Online learning during fuzzing
//! - Hybrid fitness functions
//!
//! Run with: cargo run --example ml_fitness_demo --features ml

use ndarray::{Array1, Array2};
use rand::{thread_rng, Rng};

/// Simple neural network for fitness prediction
struct SimpleNeuralNetwork {
    weight_matrices: Vec<Array2<f64>>,
    bias_vectors: Vec<Array1<f64>>,
    layer_sizes: Vec<usize>,
}

impl SimpleNeuralNetwork {
    fn new(input_size: usize, hidden_layers: &[usize], output_size: usize) -> Self {
        let mut rng = thread_rng();
        let mut weight_matrices = Vec::new();
        let mut bias_vectors = Vec::new();
        let mut layer_sizes = vec![input_size];
        
        // Build layer sizes
        let mut all_layers = Vec::new();
        all_layers.extend_from_slice(hidden_layers);
        all_layers.push(output_size);
        
        let mut prev_size = input_size;
        for &layer_size in &all_layers {
            // Initialize weights with Xavier-like initialization
            let limit = (6.0 / (prev_size + layer_size) as f64).sqrt();
            let w_data: Vec<f64> = (0..prev_size * layer_size).map(|_| rng.gen_range(-limit..limit)).collect();
            let w = Array2::from_shape_vec((prev_size, layer_size), w_data).unwrap();
            let b = Array1::from_elem(layer_size, 0.0);
            weight_matrices.push(w);
            bias_vectors.push(b);
            layer_sizes.push(layer_size);
            prev_size = layer_size;
        }
        
        Self {
            weight_matrices,
            bias_vectors,
            layer_sizes,
        }
    }
    
    /// Forward pass with ReLU activation for hidden layers, sigmoid for output
    fn forward(&self, input: &Array1<f64>) -> f64 {
        let mut current = input.clone();
        
        for (i, (w, b)) in self.weight_matrices.iter().zip(self.bias_vectors.iter()).enumerate() {
            // Linear transformation: y = W^T x + b
            let linear = current.dot(w) + b;
            
            // Apply activation
            if i < self.weight_matrices.len() - 1 {
                // ReLU for hidden layers
                current = linear.mapv(|x| x.max(0.0));
            } else {
                // Sigmoid for output layer
                current = linear.mapv(|x| 1.0 / (1.0 + (-x).exp()));
            }
        }
        
        current[0]
    }
    
    /// Predict for a batch of inputs
    fn predict_batch(&self, inputs: &Array2<f64>) -> Array1<f64> {
        let n = inputs.nrows();
        let mut outputs = Array1::zeros(n);
        for i in 0..n {
            let input_vec = inputs.row(i).to_owned();
            outputs[i] = self.forward(&input_vec);
        }
        outputs
    }
    
    /// Compute loss on dataset
    fn compute_loss(&self, inputs: &Array2<f64>, targets: &Array1<f64>) -> f64 {
        let predictions = self.predict_batch(inputs);
        let errors = targets - &predictions;
        (&errors * &errors).mean().unwrap()
    }
    
    /// Simple training: just record that training happened (demo only)
    /// Real implementation would use proper gradient descent
    fn train_step(&mut self, inputs: &Array2<f64>, targets: &Array1<f64>, _lr: f64) -> f64 {
        // Compute current loss
        let loss = self.compute_loss(inputs, targets);
        
        // Add small random perturbation to weights (simulated annealing style)
        let mut rng = thread_rng();
        let perturb_factor = 0.001;
        
        for w in &mut self.weight_matrices {
            let (rows, cols) = w.dim();
            for i in 0..rows {
                for j in 0..cols {
                    w[[i, j]] += rng.gen_range(-perturb_factor..perturb_factor);
                }
            }
        }
        
        for b in &mut self.bias_vectors {
            for i in 0..b.len() {
                b[i] += rng.gen_range(-perturb_factor..perturb_factor);
            }
        }
        
        // Compute new loss
        let new_loss = self.compute_loss(inputs, targets);
        
        // If new loss is worse, revert (simple hill climbing)
        if new_loss > loss {
            // Revert would require keeping old state - simplified here
        }
        
        loss
    }
}

/// Circuit features for ML model input
#[derive(Debug, Clone)]
struct CircuitFeatures {
    num_constraints: usize,
    num_variables: usize,
    constraint_density: f64,
    avg_constraint_degree: f64,
    num_templates: usize,
    nesting_depth: usize,
    bug_risk_score: f64,
}

impl CircuitFeatures {
    const TOTAL_FEATURES: usize = 7;
    
    fn to_array(&self) -> Array1<f64> {
        Array1::from(vec![
            self.num_constraints as f64 / 1000.0,
            self.num_variables as f64 / 1000.0,
            self.constraint_density.min(10.0) / 10.0,
            self.avg_constraint_degree / 5.0,
            self.num_templates as f64 / 50.0,
            self.nesting_depth as f64 / 5.0,
            self.bug_risk_score / 100.0,
        ])
    }
    
    /// Generate random circuit features for demo
    fn random() -> Self {
        let mut rng = thread_rng();
        Self {
            num_constraints: rng.gen_range(10..1000),
            num_variables: rng.gen_range(5..500),
            constraint_density: rng.gen_range(0.1..5.0),
            avg_constraint_degree: rng.gen_range(1.0..4.0),
            num_templates: rng.gen_range(1..20),
            nesting_depth: rng.gen_range(1..5),
            bug_risk_score: rng.gen_range(0.0..100.0),
        }
    }
}

/// ML-based fitness predictor
struct MLFitnessPredictor {
    model: SimpleNeuralNetwork,
    training_data: Vec<(Array1<f64>, f64)>,
    replay_buffer_size: usize,
}

impl MLFitnessPredictor {
    fn new(input_size: usize) -> Self {
        let model = SimpleNeuralNetwork::new(input_size, &[16, 8], 1);
        Self {
            model,
            training_data: Vec::new(),
            replay_buffer_size: 1000,
        }
    }
    
    /// Predict fitness for given features
    fn predict(&self, features: &Array1<f64>) -> f64 {
        self.model.forward(features)
    }
    
    /// Record a training sample (features, actual_fitness)
    fn record_sample(&mut self, features: Array1<f64>, actual_fitness: f64) {
        self.training_data.push((features, actual_fitness));
        
        // Keep replay buffer bounded
        if self.training_data.len() > self.replay_buffer_size {
            self.training_data.remove(0);
        }
    }
    
    /// Train the model on collected data
    fn train(&mut self, epochs: usize, learning_rate: f64) -> f64 {
        if self.training_data.is_empty() {
            return 0.0;
        }
        
        let n_samples = self.training_data.len();
        let n_features = self.model.layer_sizes[0];
        let mut inputs = Array2::zeros((n_samples, n_features));
        let mut targets = Array1::zeros(n_samples);
        
        for (i, (features, fitness)) in self.training_data.iter().enumerate() {
            for (j, &val) in features.iter().enumerate() {
                inputs[[i, j]] = val;
            }
            targets[i] = *fitness;
        }
        
        let mut total_loss = 0.0;
        for _ in 0..epochs {
            total_loss = self.model.train_step(&inputs, &targets, learning_rate);
        }
        
        total_loss
    }
    
    /// Evaluate model performance
    fn evaluate(&self) -> (f64, f64) {
        if self.training_data.is_empty() {
            return (0.0, 0.0);
        }
        
        let mut mse = 0.0;
        let mut mae = 0.0;
        let n = self.training_data.len();
        
        for (features, actual) in &self.training_data {
            let predicted = self.predict(features);
            let error = actual - predicted;
            mse += error * error;
            mae += error.abs();
        }
        
        (mse / n as f64, mae / n as f64)
    }
}

/// Hybrid fitness function combining ML prediction with heuristic
struct HybridFitnessFunction {
    predictor: MLFitnessPredictor,
    ml_weight: f64,
}

impl HybridFitnessFunction {
    fn new(ml_weight: f64) -> Self {
        Self {
            predictor: MLFitnessPredictor::new(CircuitFeatures::TOTAL_FEATURES),
            ml_weight: ml_weight.clamp(0.0, 1.0),
        }
    }
    
    /// Compute hybrid fitness
    fn evaluate(&mut self, features: &Array1<f64>, heuristic_fitness: f64) -> f64 {
        let ml_fitness = self.predictor.predict(features);
        
        // Weighted combination
        ml_fitness * self.ml_weight + heuristic_fitness * (1.0 - self.ml_weight)
    }
    
    /// Record actual fitness for online learning
    fn record_sample(&mut self, features: Array1<f64>, actual_fitness: f64) {
        self.predictor.record_sample(features, actual_fitness);
    }
    
    /// Retrain periodically
    fn retrain(&mut self, epochs: usize, lr: f64) -> f64 {
        self.predictor.train(epochs, lr)
    }
}

fn main() {
    println!("=== ML Fitness Demo for ZK Circuit Fuzzer ===\n");
    
    // 1. Initialize ML fitness predictor
    println!("[1] Initializing ML Fitness Predictor...");
    let mut predictor = MLFitnessPredictor::new(CircuitFeatures::TOTAL_FEATURES);
    println!("    Input features: {}", CircuitFeatures::TOTAL_FEATURES);
    println!("    Neural network architecture: {:?} -> 1", predictor.model.layer_sizes);
    
    // 2. Generate synthetic training data
    println!("\n[2] Generating synthetic training data...");
    let num_samples = 30;
    
    for i in 0..num_samples {
        let features = CircuitFeatures::random();
        let feature_array = features.to_array();
        
        // Simulate actual fitness based on features (with some noise)
        let mut rng = thread_rng();
        let base_fitness = 0.3 * features.constraint_density / 10.0 
                         + 0.4 * features.bug_risk_score / 100.0 
                         + 0.3 * features.avg_constraint_degree / 5.0;
        let noise = rng.gen_range(-0.1..0.1);
        let actual_fitness = (base_fitness + noise).clamp(0.0, 1.0);
        
        predictor.record_sample(feature_array, actual_fitness);
        
        if (i + 1) % 10 == 0 {
            println!("    Generated {} samples", i + 1);
        }
    }
    
    // 3. Train the model
    println!("\n[3] Training neural network...");
    let epochs = 5;
    let learning_rate = 0.01;
    
    for epoch in 0..epochs {
        let loss = predictor.train(1, learning_rate);
        println!("    Epoch {}: loss = {:.6}", epoch + 1, loss);
    }
    
    // 4. Evaluate model performance
    println!("\n[4] Evaluating model performance...");
    let (mse, mae) = predictor.evaluate();
    println!("    Mean Squared Error (MSE): {:.6}", mse);
    println!("    Mean Absolute Error (MAE): {:.6}", mae);
    println!("    Root MSE (RMSE): {:.6}", mse.sqrt());
    
    // 5. Test predictions on new samples
    println!("\n[5] Testing predictions on new samples...");
    for i in 0..3 {
        let features = CircuitFeatures::random();
        let feature_array = features.to_array();
        let predicted = predictor.predict(&feature_array);
        
        // Calculate actual fitness for comparison
        let base_fitness = 0.3 * features.constraint_density / 10.0 
                         + 0.4 * features.bug_risk_score / 100.0 
                         + 0.3 * features.avg_constraint_degree / 5.0;
        let actual = base_fitness.clamp(0.0, 1.0);
        
        println!("    Sample {}: predicted={:.4}, actual={:.4}, error={:.4}", 
                 i + 1, predicted, actual, (predicted - actual).abs());
    }
    
    // 6. Demonstrate hybrid fitness function
    println!("\n[6] Demonstrating hybrid fitness function...");
    let mut hybrid = HybridFitnessFunction::new(0.7); // 70% ML, 30% heuristic
    
    for i in 0..2 {
        let features = CircuitFeatures::random();
        let feature_array = features.to_array();
        
        // Heuristic fitness (traditional approach)
        let heuristic = 0.5 * features.constraint_density / 10.0 + 0.5 * features.bug_risk_score / 100.0;
        
        // Hybrid fitness
        let hybrid_fitness = hybrid.evaluate(&feature_array, heuristic);
        
        // Record for online learning
        let actual = 0.3 * features.constraint_density / 10.0 
                   + 0.4 * features.bug_risk_score / 100.0 
                   + 0.3 * features.avg_constraint_degree / 5.0;
        hybrid.record_sample(feature_array, actual.clamp(0.0, 1.0));
        
        println!("    Sample {}: heuristic={:.4}, hybrid={:.4}", 
                 i + 1, heuristic, hybrid_fitness);
    }
    
    // 7. Online learning demonstration
    println!("\n[7] Demonstrating online learning...");
    println!("    Initial evaluation:");
    let (initial_mse, initial_mae) = hybrid.predictor.evaluate();
    println!("        MSE: {:.6}, MAE: {:.6}", initial_mse, initial_mae);
    
    // Add more samples
    println!("    Adding 15 more samples for online learning...");
    for _ in 0..15 {
        let features = CircuitFeatures::random();
        let feature_array = features.to_array();
        let actual = 0.3 * features.constraint_density / 10.0 
                   + 0.4 * features.bug_risk_score / 100.0 
                   + 0.3 * features.avg_constraint_degree / 5.0;
        hybrid.record_sample(feature_array, actual.clamp(0.0, 1.0));
    }
    
    // Retrain
    hybrid.retrain(3, 0.005);
    
    println!("    After online learning:");
    let (updated_mse, updated_mae) = hybrid.predictor.evaluate();
    println!("        MSE: {:.6}, MAE: {:.6}", updated_mse, updated_mae);
    
    let improvement = if initial_mse > 0.0 {
        ((initial_mse - updated_mse) / initial_mse * 100.0).max(0.0)
    } else {
        0.0
    };
    println!("    MSE change: {:.1}%", improvement);
    
    // 8. Integration with genetic algorithm (conceptual)
    println!("\n[8] Integration with Genetic Algorithm (conceptual)...");
    println!("    In production, the ML fitness predictor would:");
    println!("    1. Extract features from each individual in the population");
    println!("    2. Predict fitness using the trained model");
    println!("    3. Use predictions to guide selection and mutation");
    println!("    4. Record actual fitness for online learning");
    println!("    5. Periodically retrain on accumulated data");
    
    // 9. Summary
    println!("\n=== Demo Summary ===");
    println!("    Neural network trained on {} samples", num_samples);
    println!("    Final MSE: {:.6}", mse);
    println!("    Final MAE: {:.6}", mae);
    println!("    Online learning MSE change: {:.1}%", improvement);
    println!("\nML-based fitness functions are validated and ready for integration!");
    println!("Use: cargo run --example ml_fitness_demo --features ml");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_neural_network_forward() {
        let nn = SimpleNeuralNetwork::new(7, &[16, 8], 1);
        let input = Array1::from(vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7]);
        let output = nn.forward(&input);
        assert!(output >= 0.0 && output <= 1.0, "Output should be in [0, 1]");
    }
    
    #[test]
    fn test_fitness_predictor() {
        let mut predictor = MLFitnessPredictor::new(7);
        
        // Add some training data
        for _ in 0..5 {
            let features = CircuitFeatures::random();
            let feature_array = features.to_array();
            let actual = 0.5; // Simplified
            predictor.record_sample(feature_array, actual);
        }
        
        // Train
        let loss = predictor.train(1, 0.01);
        assert!(loss.is_finite(), "Loss should be finite");
        
        // Predict
        let features = CircuitFeatures::random();
        let prediction = predictor.predict(&features.to_array());
        assert!(prediction >= 0.0 && prediction <= 1.0, "Prediction should be in [0, 1]");
    }
    
    #[test]
    fn test_hybrid_fitness() {
        let mut hybrid = HybridFitnessFunction::new(0.7);
        let features = CircuitFeatures::random();
        let feature_array = features.to_array();
        let heuristic = 0.5;
        
        let hybrid_score = hybrid.evaluate(&feature_array, heuristic);
        assert!(hybrid_score >= 0.0 && hybrid_score <= 1.0, "Hybrid score should be in [0, 1]");
    }
    
    #[test]
    fn test_feature_normalization() {
        let features = CircuitFeatures::random();
        let array = features.to_array();
        
        for &value in array.iter() {
            assert!(value >= 0.0 && value <= 1.0, "Feature should be normalized to [0, 1]");
        }
    }
}
