"""
Script to create mock ONNX models for telemetry_ai module.
Creates:
1. slm.onnx - Decision model (270 inputs -> 7 outputs)
2. embedder.onnx - Embedding model (variable input -> 128-dim output)
"""

import numpy as np
import onnx
from onnx import helper, TensorProto
import os

def create_slm_model(output_path: str):
    """Create the SLM (decision) model: 269 inputs -> 7 outputs (removed fiveg_signal)"""
    
    # Input: (batch_size, 269) - 13 numeric + 128 embed_current + 128 embed_context = 269
    input_tensor = helper.make_tensor_value_info(
        'input',
        TensorProto.FLOAT,
        [1, 269]  # batch_size=1, 269 features (removed fiveg_signal)
    )
    
    # Output: (batch_size, 7)
    # [route, severity, p2_enable, congestion, wfq_p0, wfq_p1, wfq_p2]
    output_tensor = helper.make_tensor_value_info(
        'output',
        TensorProto.FLOAT,
        [1, 7]
    )
    
    # Create a simple linear transformation: output = input @ weights + bias
    # Weight matrix: (269, 7) - updated for 269 features
    np.random.seed(42)  # For reproducibility
    weights = np.random.randn(269, 7).astype(np.float32) * 0.01
    bias = np.array([0.5, 0.3, 0.4, 0.2, 50.0, 30.0, 20.0], dtype=np.float32)
    
    # Create weight and bias tensors
    weights_tensor = helper.make_tensor(
        'weights',
        TensorProto.FLOAT,
        [269, 7],  # Updated for 269 features
        weights.flatten().tolist()
    )
    
    bias_tensor = helper.make_tensor(
        'bias',
        TensorProto.FLOAT,
        [7],
        bias.tolist()
    )
    
    # Create nodes: MatMul -> Add -> Apply activations
    matmul_node = helper.make_node(
        'MatMul',
        inputs=['input', 'weights'],
        outputs=['matmul_out']
    )
    
    add_node = helper.make_node(
        'Add',
        inputs=['matmul_out', 'bias'],
        outputs=['add_out']
    )
    
    # Apply sigmoid to first 4 outputs (route, severity, p2_enable, congestion)
    # For route, we'll use a different approach - use sigmoid then scale
    sigmoid_node = helper.make_node(
        'Sigmoid',
        inputs=['add_out'],
        outputs=['sigmoid_out']
    )
    
    # Create scale factors for weights (last 3 outputs)
    scale_100 = helper.make_tensor('scale_100', TensorProto.FLOAT, [], [100.0])
    
    # Extract individual outputs using Slice operations (simpler than Split)
    # For route: use first output, apply sigmoid, then scale to 0-3 range
    route_slice = helper.make_tensor('route_idx', TensorProto.INT64, [1], [0])
    route_slice_end = helper.make_tensor('route_idx_end', TensorProto.INT64, [1], [1])
    route_axis = helper.make_tensor('route_axis', TensorProto.INT64, [1], [1])
    
    route_slice_node = helper.make_node(
        'Slice',
        inputs=['sigmoid_out', 'route_idx', 'route_idx_end', 'route_axis'],
        outputs=['route_raw']
    )
    
    # Scale route to 0-3 range
    route_scale = helper.make_tensor('route_scale', TensorProto.FLOAT, [], [3.0])
    route_mul = helper.make_node('Mul', inputs=['route_raw', 'route_scale'], outputs=['route_scaled'])
    
    # For severity: second output
    severity_idx = helper.make_tensor('severity_idx', TensorProto.INT64, [1], [1])
    severity_idx_end = helper.make_tensor('severity_idx_end', TensorProto.INT64, [1], [2])
    severity_slice_node = helper.make_node(
        'Slice',
        inputs=['sigmoid_out', 'severity_idx', 'severity_idx_end', 'route_axis'],
        outputs=['severity_out']
    )
    
    # For p2_enable: third output
    p2_idx = helper.make_tensor('p2_idx', TensorProto.INT64, [1], [2])
    p2_idx_end = helper.make_tensor('p2_idx_end', TensorProto.INT64, [1], [3])
    p2_slice_node = helper.make_node(
        'Slice',
        inputs=['sigmoid_out', 'p2_idx', 'p2_idx_end', 'route_axis'],
        outputs=['p2_out']
    )
    
    # For congestion: fourth output
    cong_idx = helper.make_tensor('cong_idx', TensorProto.INT64, [1], [3])
    cong_idx_end = helper.make_tensor('cong_idx_end', TensorProto.INT64, [1], [4])
    cong_slice_node = helper.make_node(
        'Slice',
        inputs=['sigmoid_out', 'cong_idx', 'cong_idx_end', 'route_axis'],
        outputs=['cong_out']
    )
    
    # For weights: use ReLU on last 3 outputs, then scale
    weight_start_idx = helper.make_tensor('weight_start', TensorProto.INT64, [1], [4])
    weight_end_idx = helper.make_tensor('weight_end', TensorProto.INT64, [1], [7])
    weight_slice_node = helper.make_node(
        'Slice',
        inputs=['add_out', 'weight_start_idx', 'weight_end_idx', 'route_axis'],
        outputs=['weight_raw']
    )
    
    relu_node = helper.make_node('Relu', inputs=['weight_raw'], outputs=['weight_relu'])
    weight_mul = helper.make_node('Mul', inputs=['weight_relu', 'scale_100'], outputs=['weight_scaled'])
    
    # Concatenate all outputs
    concat_node = helper.make_node(
        'Concat',
        inputs=['route_scaled', 'severity_out', 'p2_out', 'cong_out', 'weight_scaled'],
        outputs=['output'],
        axis=1
    )
    
    # Actually, let's simplify - just use the sigmoid output directly for first 4,
    # and ReLU+scale for last 3
    # Let's create a simpler version
    weight_start = helper.make_tensor('weight_start', TensorProto.INT64, [1], [4])
    weight_end = helper.make_tensor('weight_end', TensorProto.INT64, [1], [7])
    
    # Simplified approach: just use the add output with appropriate activations
    # Split into two parts: first 4 (sigmoid) and last 3 (relu)
    first_four_start = helper.make_tensor('ff_start', TensorProto.INT64, [1], [0])
    first_four_end = helper.make_tensor('ff_end', TensorProto.INT64, [1], [4])
    
    first_four = helper.make_node(
        'Slice',
        inputs=['add_out', 'ff_start', 'ff_end', 'route_axis'],
        outputs=['first_four']
    )
    
    last_three_start = helper.make_tensor('lt_start', TensorProto.INT64, [1], [4])
    last_three_end = helper.make_tensor('lt_end', TensorProto.INT64, [1], [7])
    
    last_three = helper.make_node(
        'Slice',
        inputs=['add_out', 'lt_start', 'lt_end', 'route_axis'],
        outputs=['last_three']
    )
    
    sigmoid_four = helper.make_node('Sigmoid', inputs=['first_four'], outputs=['sigmoid_four'])
    relu_three = helper.make_node('Relu', inputs=['last_three'], outputs=['relu_three'])
    scale_three = helper.make_node('Mul', inputs=['relu_three', 'scale_100'], outputs=['scaled_three'])
    
    concat_final = helper.make_node(
        'Concat',
        inputs=['sigmoid_four', 'scaled_three'],
        outputs=['output'],
        axis=1
    )
    
    # Create the graph
    graph = helper.make_graph(
        nodes=[
            matmul_node,
            add_node,
            first_four,
            last_three,
            sigmoid_four,
            relu_three,
            scale_three,
            concat_final
        ],
        name='slm_model',
        inputs=[input_tensor],
        outputs=[output_tensor],
        initializer=[weights_tensor, bias_tensor, scale_100, first_four_start, first_four_end, last_three_start, last_three_end, route_axis]
    )
    
    # Create the model with IR version 10 (max supported by ort)
    model = helper.make_model(graph, producer_name='telemetry_ai', ir_version=10)
    model.opset_import[0].version = 13
    
    # Save the model
    onnx.checker.check_model(model)
    onnx.save_model(model, output_path)
    print(f"Created {output_path} successfully!")


def create_embedder_model(output_path: str):
    """Create the embedder model: variable input -> 128-dim output"""
    
    # Input: (batch_size, sequence_length) - for simplicity, we'll use (1, 1024) as example
    # In practice, this would be variable length, but ONNX needs fixed shapes
    input_tensor = helper.make_tensor_value_info(
        'input',
        TensorProto.FLOAT,
        [1, 1024]  # batch_size=1, 1024 features (chunk size)
    )
    
    # Output: (batch_size, 128)
    output_tensor = helper.make_tensor_value_info(
        'output',
        TensorProto.FLOAT,
        [1, 128]
    )
    
    # Create a simple embedding: Dense layer 1024 -> 128
    np.random.seed(42)
    weights = np.random.randn(1024, 128).astype(np.float32) * 0.1
    bias = np.random.randn(128).astype(np.float32) * 0.1
    
    weights_tensor = helper.make_tensor(
        'embed_weights',
        TensorProto.FLOAT,
        [1024, 128],
        weights.flatten().tolist()
    )
    
    bias_tensor = helper.make_tensor(
        'embed_bias',
        TensorProto.FLOAT,
        [128],
        bias.tolist()
    )
    
    # MatMul -> Add -> Tanh (for normalization)
    matmul_node = helper.make_node(
        'MatMul',
        inputs=['input', 'embed_weights'],
        outputs=['matmul_out']
    )
    
    add_node = helper.make_node(
        'Add',
        inputs=['matmul_out', 'embed_bias'],
        outputs=['add_out']
    )
    
    tanh_node = helper.make_node(
        'Tanh',
        inputs=['add_out'],
        outputs=['output']
    )
    
    # Create the graph
    graph = helper.make_graph(
        nodes=[matmul_node, add_node, tanh_node],
        name='embedder_model',
        inputs=[input_tensor],
        outputs=[output_tensor],
        initializer=[weights_tensor, bias_tensor]
    )
    
    # Create the model with IR version 10 (max supported by ort)
    model = helper.make_model(graph, producer_name='telemetry_ai', ir_version=10)
    model.opset_import[0].version = 13
    
    # Save the model
    onnx.checker.check_model(model)
    onnx.save_model(model, output_path)
    print(f"Created {output_path} successfully!")


if __name__ == '__main__':
    # Create models directory if it doesn't exist
    os.makedirs('models', exist_ok=True)
    
    # Create the models
    create_slm_model('models/slm.onnx')
    create_embedder_model('models/embedder.onnx')
    
    print("\nAll ONNX models created successfully!")
    print("   - models/slm.onnx (270 inputs -> 7 outputs)")
    print("   - models/embedder.onnx (1024 inputs -> 128 outputs)")
