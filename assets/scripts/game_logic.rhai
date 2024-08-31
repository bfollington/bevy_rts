// This function will be called when the script is loaded
fn init() {
    print_message("Hello from Rhai script!");
}

// Call the init function
init();

fn update() {
    let desired_velocity = 0.0;

    read_input("A").then(|left| {
        if left {
            print_message("Left");
            desired_velocity -= 1.0;
        }

        read_input("D").then(|right| {
            if right {
                print_message("Right");
                desired_velocity += 1.0;
            }

            set_desired_velocity(entity, desired_velocity, 0.0);
        })
    });

    read_input("Space").then(|jump| {
        if jump {
            print_message("Jump");
            queue_jump(entity);
        }
    });
}
