# Build and test
echo "Building Morris with fixes..."
cargo clean
cargo build

if [ $? -eq 0 ]; then
    echo "✅ Build successful! Checking for warnings..."
    
    if cargo check 2>&1 | grep -q "warning:"; then
        echo "❌ Still have warnings:"
        cargo check 2>&1 | grep "warning:"
    else
        echo "✅ No warnings found!"
        
        # Test the fixes
        echo -e "\nTesting fixes..."
        
        # Test 1: Function calls
        echo -e "\nTest 1: Function calls"
        echo -e "set text = \"hello hello world\"\nset c = count(text, \"hello\")\nwriteout(Count: {c})\nexit" | ./target/debug/morris 2>&1 | grep -q "Count: 2" && echo "✅ count() works" || echo "❌ count() failed"
        
        # Test 2: Now function
        echo -e "\nTest 2: now() function"
        echo -e "set ts = now()\nwriteout(Timestamp: {ts})\nexit" | ./target/debug/morris 2>&1 | grep -q "Timestamp: 2" && echo "✅ now() works" || echo "❌ now() failed"
        
        # Test 3: String interpolation
        echo -e "\nTest 3: String interpolation"
        echo -e "set name = \"Morris\"\nset version = 0.5\nset msg = \"Hello from \" + name + \" v\" + version\nwriteout({msg})\nexit" | ./target/debug/morris 2>&1 | grep -q "Hello from Morris v0.5" && echo "✅ String concat works" || echo "❌ String concat failed"
        
        echo -e "\n✅ All fixes applied!"
        echo -e "\nTry the fixed examples:"
        echo -e "  ./target/debug/morris examples/data_pipeline_fixed.msh"
        echo -e "  ./target/debug/morris examples/config_manager_fixed.msh"
    fi
else
    echo "❌ Build failed"
    exit 1
fi