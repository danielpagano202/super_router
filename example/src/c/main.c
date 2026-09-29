#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <windows.h>

#define MAX_LINE_LENGTH 4096

// Helper function to simulate Python's os.path.dirname(os.path.abspath(__file__))
void get_script_dir(char* buffer, size_t buffer_size) {
    GetModuleFileNameA(NULL, buffer, buffer_size);
    char* last_slash = strrchr(buffer, '\\');
    if (last_slash != NULL) {
        *last_slash = '\0'; // Truncate the file name to leave only the directory path
    }
}

int main() {
    char line[MAX_LINE_LENGTH];
    char script_dir[MAX_PATH];
    char html_path[MAX_PATH];
    char json_response[MAX_LINE_LENGTH * 2];

    // Loop through each line coming from Rust's stdin
    while (fgets(line, sizeof(line), stdin) != NULL) {
        // Equivalent to python's try block
        if (1) { 
            // Get the directory where this executable lives
            get_script_dir(script_dir, sizeof(script_dir));
            
            // Assemble absolute path: script_dir + \index.html
            snprintf(html_path, sizeof(html_path), "%s\\index.html", script_dir);
            
            // Format backslashes for JSON compatibility (escaped to \\)
            char escaped_html_path[MAX_PATH * 2] = {0};
            char* dst = escaped_html_path;
            for (char* src = html_path; *src != '\0'; src++) {
                if (*src == '\\') {
                    *dst++ = '\\';
                    *dst++ = '\\';
                } else {
                    *dst++ = *src;
                }
            }

            // Convert data layout to a clean single-line JSON string
            snprintf(json_response, sizeof(json_response),
                     "{\"responseType\": \"htmlfile\", \"responseCode\": 200, \"data\": \"%s\"}",
                     escaped_html_path);
            
            // CRUCIAL: print the JSON line and force flush it to Rust immediately
            printf("%s\n", json_response);
            fflush(stdout); 
        } 
        // In case of parsing memory issues or catastrophic exceptions
        else {
            printf("ERROR: Processing failed\n");
            fflush(stdout);
        }
    }

    return 0;
}
