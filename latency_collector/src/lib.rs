pub struct LatencyCollector {
    min_latency: u64,
    max_latency: u64,
    total_latency: u64,
    total_msgs: u64,
    avg_latency: u64,
    less_than_400: u64,
    less_than_800: u64,
    less_than_1000: u64,
    less_than_1200: u64,
    less_than_1500: u64,
    less_than_1800: u64,
    less_than_2000: u64,
    more_than_2000: u64,
}

impl LatencyCollector {
    pub fn new() -> Self {
        LatencyCollector {
            min_latency: u64::MAX,
            max_latency: u64::MIN,
            total_latency: 0,
            total_msgs: 0,
            avg_latency: 0,
            less_than_400: 0,
            less_than_800: 0,
            less_than_1000: 0,
            less_than_1200: 0,
            less_than_1500: 0,
            less_than_1800: 0,
            less_than_2000: 0,
            more_than_2000: 0,
        }
    }

    pub fn collect_data(&mut self, exact_timestamp_ms: u64, received_timestamp_ms: u64) {
        let latency = self.calculate_latency(exact_timestamp_ms, received_timestamp_ms);

        if latency < self.min_latency {
            self.min_latency = latency;
        }
        if latency > self.max_latency {
            self.max_latency = latency;
        }

        self.total_latency += latency;
        self.total_msgs += 1;
        self.avg_latency = self.total_latency / self.total_msgs;

        match latency {
            l if l < 400 => self.less_than_400 += 1,
            l if l < 800 => self.less_than_800 += 1,
            l if l < 1000 => self.less_than_1000 += 1,
            l if l < 1200 => self.less_than_1200 += 1,
            l if l < 1500 => self.less_than_1500 += 1,
            l if l < 1800 => self.less_than_1800 += 1,
            l if l < 2000 => self.less_than_2000 += 1,
            _ => self.more_than_2000 += 1,
        }
    }

    fn calculate_latency(&self, transaction_time: u64, time_received: u64) -> u64 {
        time_received.saturating_sub(transaction_time)
    }
    pub fn generate_report(&self) {
        println!("**********************************************\n");
        println!("*  Min Latency: {}", self.min_latency);
        println!("*  Max Latency: {}", self.max_latency);
        let avg_latency = if self.total_msgs > 0 {
            self.total_latency as f64 / self.total_msgs as f64
        } else {
            0.0
        };
        println!("*  Average Latency: {:.2}", avg_latency);
        println!("*  Total Msgs: {}", self.total_msgs);
        println!(
            "*  0-399ms: {} | {:.2} %",
            self.less_than_400,
            (self.less_than_400 as f64 / self.total_msgs as f64) * 100.0
        );
        println!(
            "*  400-799ms: {} | {:.2} %",
            self.less_than_800,
            (self.less_than_800 as f64 / self.total_msgs as f64) * 100.0
        );
        println!(
            "*  800-999ms: {} | {:.2} %",
            self.less_than_1000,
            (self.less_than_1000 as f64 / self.total_msgs as f64) * 100.0
        );
        println!(
            "*  1000-1199ms: {} | {:.2} %",
            self.less_than_1200,
            (self.less_than_1200 as f64 / self.total_msgs as f64) * 100.0
        );
        println!(
            "*  1200-1499ms: {} | {:.2} %",
            self.less_than_1500,
            (self.less_than_1500 as f64 / self.total_msgs as f64) * 100.0
        );
        println!(
            "*  1500-1799ms: {} | {:.2} %",
            self.less_than_1800,
            (self.less_than_1800 as f64 / self.total_msgs as f64) * 100.0
        );
        println!(
            "*  1800-2000ms: {} | {:.2} %",
            self.less_than_2000,
            (self.less_than_2000 as f64 / self.total_msgs as f64) * 100.0
        );
        println!(
            "*  2000ms+: {} | {:.2} %",
            self.more_than_2000,
            (self.more_than_2000 as f64 / self.total_msgs as f64) * 100.0
        );
        println!("\n**********************************************");
    }
}
