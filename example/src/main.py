import sys
import json
import os

# Loop through each line coming from Rust's stdin
for line in sys.stdin:    
    try:
        # Get the directory where this script lives
        script_dir = os.path.dirname(os.path.abspath(__file__))
        html_path = os.path.join(script_dir, "index.html")
        
        # 1. Fixed key syntax ("type") and absolute path assembly
        data = {
           "response_type": "htmlfile",
           "response_code": 200,
           "data": html_path
        }
        
        # 2. Convert dictionary to a clean single-line JSON string
        json_response = json.dumps(data)
        
        # 3. CRUCIAL: print the JSON line and force flush it to Rust immediately
        print(json_response, flush=True)
        
    except Exception as e:
        # If anything breaks, print to stdout so Rust can see the error instead of hanging
        print(f"ERROR: {str(e)}", flush=True)
