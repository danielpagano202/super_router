import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.io.File;
import java.util.Map;

public class main {
    public static void main(String[] args) {
        // Wrap System.in to read line-by-line efficiently
        try (BufferedReader reader = new BufferedReader(new InputStreamReader(System.in))) {
            String line;
            
            while ((line = reader.readLine()) != null) {

                try {
                    // Get the directory where this compiled class or JAR lives
                    String scriptDir = new File(main.class.getProtectionDomain()
                            .getCodeSource()
                            .getLocation()
                            .toURI())
                            .getParent();
                    
                    // Construct absolute path to index.html (equivalent to os.path.join)
                    File htmlFile = new File(scriptDir, "index.html");
                    String htmlPath = htmlFile.getAbsolutePath();

                    // 1 & 2. Create the data map and format it as a single-line JSON string
                    // Note: We use escaped quotes manually to avoid heavy external dependencies like Jackson/Gson
                    String jsonResponse = String.format(
                        "{\"responseType\":\"%s\",\"responseCode\":%d,\"data\":\"%s\"}",
                        "text", 200, "hello"
                    );

                    // 3. CRUCIAL: Print the JSON line and immediately force flush it to Rust
                    System.out.println(jsonResponse);
                    System.out.flush();

                } catch (Exception e) {
                    // If anything breaks, print to stdout so Rust can see the error instead of hanging
                    System.out.println("ERROR: " + e.getMessage());
                    System.out.flush();
                }
            }
        } catch (Exception e) {
            System.out.println("ERROR: Failed to initialize stdin reader: " + e.getMessage());
            System.out.flush();
        }
    }
}
