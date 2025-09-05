#!/bin/bash

# Store original governor and frequency limits
ORIGINAL_GOVERNOR=$(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor)
ORIGINAL_MIN_FREQ=$(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_min_freq)
ORIGINAL_MAX_FREQ=$(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_max_freq)

echo "Original governor: $ORIGINAL_GOVERNOR"
echo "Original min frequency: $ORIGINAL_MIN_FREQ kHz"
echo "Original max frequency: $ORIGINAL_MAX_FREQ kHz"

# Check available governors
echo "Available governors:"
cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_available_governors

# Check current min/max frequencies
echo "Current min frequency: $(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_min_freq) kHz"
echo "Current max frequency: $(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_max_freq) kHz"

# Set governor to performance for maximum performance
echo performance | sudo tee /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor
echo "Set governor to: performance"

# Lock frequency by setting min and max to the same value
# Use 4.0 GHz (4000000 kHz) which is within your supported range (800MHz - 5.6GHz)
TARGET_FREQ=4000000
echo $TARGET_FREQ | sudo tee /sys/devices/system/cpu/cpu0/cpufreq/scaling_min_freq
echo $TARGET_FREQ | sudo tee /sys/devices/system/cpu/cpu0/cpufreq/scaling_max_freq
echo "Locked frequency to: $TARGET_FREQ kHz (4.0 GHz)"

# Verify current settings
echo "Current governor: $(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor)"
echo "Current frequency: $(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq) kHz"
echo "Current min frequency: $(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_min_freq) kHz"
echo "Current max frequency: $(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_max_freq) kHz"

# Run benchmark with full PATH
echo "Running benchmark..."

# Get the current user (who called sudo)
ACTUAL_USER=${SUDO_USER:-$USER}

echo "Running benchmark as user: $ACTUAL_USER"
echo "CPU settings are locked to 4.0 GHz on core 0"

# Parse arguments
SNAKE_COUNT=1000  # Default snake count
COLD_CACHE=false

for arg in "$@"; do
    case $arg in
        --cold-cache)
            COLD_CACHE=true
            ;;
        --snakes=*)
            SNAKE_COUNT="${arg#*=}"
            ;;
        *)
            # If it's just a number, treat it as snake count
            if [[ $arg =~ ^[0-9]+$ ]]; then
                SNAKE_COUNT=$arg
            fi
            ;;
    esac
done

echo "Running benchmark with $SNAKE_COUNT snakes"

# Check if we want cold cache measurement
if [ "$COLD_CACHE" = true ]; then
    echo "=== COLD CACHE BENCHMARK ==="
    echo "Clearing kernel caches..."
    echo 3 | sudo tee /proc/sys/vm/drop_caches > /dev/null
    
    echo "Clearing CPU caches on core 0 with memory-intensive operations..."
    # Write large amount of data on core 0 to evict our benchmark data from CPU caches
    sudo -u $ACTUAL_USER taskset -c 0 dd if=/dev/zero of=/tmp/cache_clear bs=1M count=2000 2>/dev/null
    sudo -u $ACTUAL_USER taskset -c 0 rm /tmp/cache_clear
    
    # Hmm not sure if this is actually helping clearing cache, but care about hot cache anyway
    echo "Running cold cache benchmark..."
    sudo -u $ACTUAL_USER taskset -c 0 /home/boopop/.cargo/bin/cargo bench --bench game_bench hot_path/${SNAKE_COUNT}_snakes
else
    echo "=== WARM CACHE BENCHMARK ==="
    echo "Running warm-up benchmark (discarding result)..."
    sudo -u $ACTUAL_USER taskset -c 0 /home/boopop/.cargo/bin/cargo bench --bench game_bench hot_path/${SNAKE_COUNT}_snakes > /dev/null 2>&1
    
    echo "Running actual benchmark (measuring this result)..."
    sudo -u $ACTUAL_USER taskset -c 0 /home/boopop/.cargo/bin/cargo bench --bench game_bench hot_path/${SNAKE_COUNT}_snakes
fi

echo "Benchmark completed"

# Restore original governor
echo "Restoring governor to: $ORIGINAL_GOVERNOR"
echo $ORIGINAL_GOVERNOR | sudo tee /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor

# Restore original min/max frequencies
echo "Restoring frequency limits to original values"
echo $ORIGINAL_MIN_FREQ | sudo tee /sys/devices/system/cpu/cpu0/cpufreq/scaling_min_freq
echo $ORIGINAL_MAX_FREQ | sudo tee /sys/devices/system/cpu/cpu0/cpufreq/scaling_max_freq

echo "Benchmark complete, governor and frequency limits restored"