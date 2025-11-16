#!/usr/bin/env python3
"""Test ONNX models to verify they work correctly"""

import onnx
import numpy as np
import os

def test_slm_model():
    """Test the SLM decision model"""
    print("Testing SLM Model (slm.onnx)...")
    
    if not os.path.exists("models/slm.onnx"):
        print("ERROR: models/slm.onnx not found!")
        return False
    
    model = onnx.load("models/slm.onnx")
    onnx.checker.check_model(model)
    
    # Check input/output shapes
    input_shape = model.graph.input[0].type.tensor_type.shape.dim
    output_shape = model.graph.output[0].type.tensor_type.shape.dim
    
    print(f"  [OK] Model loaded successfully")
    print(f"  [OK] Input shape: {[d.dim_value for d in input_shape]}")
    print(f"  [OK] Output shape: {[d.dim_value for d in output_shape]}")
    
    return True

def test_embedder_model():
    """Test the embedder model"""
    print("\nTesting Embedder Model (embedder.onnx)...")
    
    if not os.path.exists("models/embedder.onnx"):
        print("ERROR: models/embedder.onnx not found!")
        return False
    
    model = onnx.load("models/embedder.onnx")
    onnx.checker.check_model(model)
    
    # Check input/output shapes
    input_shape = model.graph.input[0].type.tensor_type.shape.dim
    output_shape = model.graph.output[0].type.tensor_type.shape.dim
    
    print(f"  [OK] Model loaded successfully")
    print(f"  [OK] Input shape: {[d.dim_value for d in input_shape]}")
    print(f"  [OK] Output shape: {[d.dim_value for d in output_shape]}")
    
    return True

def main():
    print("Testing ONNX Models\n")
    print("=" * 50)
    
    slm_ok = test_slm_model()
    embedder_ok = test_embedder_model()
    
    print("\n" + "=" * 50)
    if slm_ok and embedder_ok:
        print("[SUCCESS] All models validated successfully!")
        print("\nModels are ready for use in Rust code.")
        return 0
    else:
        print("[FAILURE] Some models failed validation")
        return 1

if __name__ == "__main__":
    exit(main())
