use serde_json::json;

fn main() {
    // Test case 1: This works - simple nested selector
    let rule1 = r#"
    A **user** is valid if __prop__ of **top.second** is equal to "test".
    "#;
    
    let json1 = json!({
        "top": {
            "second": {
                "prop": "test"
            }
        }
    });
    
    println!("Test 1 - Simple nested selector:");
    println!("Rule: {}", rule1.trim());
    println!("JSON: {}", serde_json::to_string_pretty(&json1).unwrap());
    
    // Test case 2: This fails - "in the" syntax
    let rule2 = r#"
    A **student** is valid 
      if the __criminal_background_clear__ of the **background.criminal** in the **student** is equal to true.
    "#;
    
    let json2 = json!({
        "student": {
            "background": {
                "criminal": {
                    "criminal_background_clear": true
                }
            }
        }
    });
    
    println!("\nTest 2 - 'in the' syntax:");
    println!("Rule: {}", rule2.trim());
    println!("JSON: {}", serde_json::to_string_pretty(&json2).unwrap());
    
    // Test case 3: Compare with working syntax
    let rule3 = r#"
    A **student** is valid 
      if the __research alignment__ of the **student.advisor.fit** is equal to true.
    "#;
    
    let json3 = json!({
        "student": {
            "advisor": {
                "fit": {
                    "research_alignment": true
                }
            }
        }
    });
    
    println!("\nTest 3 - Working nested path:");
    println!("Rule: {}", rule3.trim());
    println!("JSON: {}", serde_json::to_string_pretty(&json3).unwrap());
    
    // Show the expected parsing result for Test 2
    println!("\n=== Analysis ===");
    println!("In Test 2, the path should resolve as:");
    println!("1. Start at root");
    println!("2. Navigate to 'student' (from 'in the **student**')");
    println!("3. Navigate to 'background.criminal' (from '**background.criminal**')");
    println!("4. Get property 'criminal_background_clear'");
    println!("Expected path: $.student.background.criminal.criminal_background_clear");
    
    println!("\nBut it's likely being parsed as:");
    println!("1. Try to find 'background.criminal' at root level (fails!)");
    println!("2. Cannot continue because selector not found");
}