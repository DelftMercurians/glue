# glue
[Delft Mercurians](https://delftmercurians.nl/)

This code forms a bridge between the BaseStation (running on C++) and the MotherShip (running Rust). It allows us to read out various robot status information and send commands to the robot.

## Installing
Add the following to your Cargo.toml
```TOML
glue = { git = "https://github.com/DelftMercurians/glue.git", tag = "v0.2.0" }
```
Make sure to update the version tag when new changes are pushed.


## Usage

Start the monitor using the following command. This spawns a new thread that takes care of serial communication in the background.
```Rust
let monitor = glue::Monitor::start();
```

To get a list of available serial ports, run the following. If filter is set to true, only serial ports which might be base stations are shown (filtered by USB VID and PID). It is recommended to always use `filter = true`.

```Rust
glue::Serial::list_ports(filter : bool) -> Vec<String>
```

Then, to connect to a BaseStation use the following. The first function allows you to connect to an arbitrary port, the latter connects to the first found port. The Result indicates if the connection was successful.
```Rust
monitor.connect_to(port : &str) -> Result<(), ()>
monitor.connect_to_first() -> Result<(), ()>
```

The following function can be used to detect whether the BaseStation is still connected or has been disconnected.
```Rust
monitor.is_connected() -> bool
```

All serial communication runs in the background, so there is no need to run polling functions or worry about buffers overflowing if nothing is called.

Use the following to disconnect from the base station. When the monitor goes out of scope, this happens automatically.
```Rust
monitor.disconnect() -> Result<(), ()>
```

### Receiving Data from Robot

To receive data from the robot, run the following. This is a snapshot of the current known robot state. Note that this function may return `None` if no BaseStation is connected, or if a mutex lock cannot be aquired.
```Rust
monitor.get_robots() -> Option<[glue::Robot; glue::MAX_NUM_ROBOTS]>
```

The index to the returned array is the SSL ID of the robot. The `Robot` struct has many useful functions to read out the robot state:
```Rust
robot.time_since_update() -> Option<std::time::Duration>
robot.is_online() -> bool

robot.primary_status() -> Option<glue::HG_Status>
robot.kicker_status() -> Option<glue::HG_Status>
robot.imu_status() -> Option<glue::HG_Status>
robot.fan_status() -> Option<glue::HG_Status>

robot.motor_statuses() -> Option<[glue::HG_Status; 5]>
robot.motor_speeds() -> Option<[f32; 5]> // [rad/s]
robot.motor_temperatures() -> Option<[f32; 5]> // [deg C]

robot.breakbeam_ball_detected() -> Option<bool>

robot.pack_voltages() -> Option<[f32; 2]> // [V]

robot.kicker_cap_voltage() -> Option<f32> // [V]
robot.kicker_temperature() -> Option<f32>  // [deg C]
```


### Sending Data to Robot
Currently, only one command to the robot is supported. Use it as follows:
```Rust
let mut commands = [None; glue::MAX_NUM_ROBOTS];
let robot_id : u8 = 2; // SSL ID
commands[robot_id] = Some(glue::Radio_Command {
    speed: glue::HG_Pose {
        x: 0.0, // X speed (f32) [m/s]
        y: 0.0, // Y speed (f32) [m/s]
        z: 0.0, // Z speed (f32) [rad/s]
    },
    dribbler_speed: 6000.0,    // Dribbler speed (f32) [rad/s]
    robot_command: glue::Radio_RobotCommand::NONE,   // Kicker command (glue::Radio_RobotCommand enum)
    _pad: [0, 0, 0],    // padding, leave zero
    kick_time: 0.0, // Kick time (f32) [ms]
    fan_speed: 0.0, // Fan speed (f32) [%]
});
let result = monitor.send(commands);
```

Also note that currently, only one robot should be addressed at a time. This will be fixed in future revisions.


### Receiving Status from BaseStation
The BaseStation also emits info packets, which can be used to check if everything is still well. They can be used as follows. The developer experience is not as refined here, so there may be issues.
```Rust
if let glue::Stamped::Have(timestamp, base_info) = monitor.get_base_info() {
    let time_elapsed : std::time::Duration = timestamp.elapsed(); // time since basestation posted an update
    let version_string : String = base_info.version.version_to_string();
    let protocol_version_string : String = base_info.version.protocol_version_to_string();
    let protocol_version_match : bool = base_info.version.protcol_version_matches();  // True if the basestation and glue are using the same protocol version
    let num_radios : u8 = base_info.num_radios;  // Number of radios that can theoretically be installed on this station
    let max_robots : u8 = base_info.max_robots;  // Maximum number of robots that this station can communicate with (Always start at SSL id 0)
    let radios_online : u16 = base_info.radios_online;  // Bitfield signifying which of the radios are working (LSB = radio 0)
}
```