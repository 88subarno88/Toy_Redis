import redis
import time
import json

print("Connecting to Rust Redis Engine...")
# Connecting to IPv4 localhost just in case!
r = redis.Redis(host='127.0.0.1', port=6379, decode_responses=True)

metrics = {}

try:
    start = time.time()
    r.ping()
    metrics['Ping'] = (time.time() - start) * 1000

    print("Writing 10,000 keys...")
    start = time.time()
    for i in range(10000):
        r.set(f"test:{i}", "data")
    metrics['10k Writes'] = (time.time() - start) * 1000

    print("Reading 10,000 keys...")
    start = time.time()
    for i in range(10000):
        r.get(f"test:{i}")
    metrics['10k Reads'] = (time.time() - start) * 1000

    print("Deleting 10,000 keys...")
    start = time.time()
    for i in range(10000):
        r.delete(f"test:{i}")
    metrics['10k Deletes'] = (time.time() - start) * 1000

except Exception as e:
    print(f"Error: {e}")
    exit(1)

# Barebones HTML template with just the Chart.js canvas
html_template = f"""
<!DOCTYPE html>
<html>
<head>
    <title>Redis Graph</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
</head>
<body>
    <h2>Rust Database Performance (Milliseconds)</h2>
    <div style="width: 600px;">
        <canvas id="myChart"></canvas>
    </div>

    <script>
        const data = {json.dumps(metrics)};
        new Chart(document.getElementById('myChart'), {{
            type: 'bar',
            data: {{
                labels: Object.keys(data),
                datasets: [{{
                    label: 'Time (ms)',
                    data: Object.values(data),
                    borderWidth: 1
                }}]
            }},
            options: {{ scales: {{ y: {{ beginAtZero: true }} }} }}
        }});
    </script>
</body>
</html>
"""

# Save it specifically as graph.html
with open("graph.html", "w") as f:
    f.write(html_template)

print("Done! Saved to graph.html")