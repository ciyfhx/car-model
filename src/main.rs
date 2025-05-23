use bevy::app::{App, Startup};
use bevy::prelude::*;

const SIZE: Vec2 = Vec2::new(1200.0, 800.0);

#[derive(Resource)]
struct SpeedTimer(Timer);

#[derive(Component)]
struct Car{
    mov_speed: f32,
    max_mov_speed: f32,
    yaw_rad: f32,
    steering_rad: f32,
    max_steering_rad: f32,
    accel_speed: f32,
    brake_speed: f32,
    wheel_base: f32,
    track: f32,
}

impl Default for Car {
    fn default() -> Car {
        Car {
            mov_speed: 0.0,
            max_mov_speed: 400.0,
            yaw_rad: 0.0,
            steering_rad: 0.0,
            max_steering_rad: f32::to_radians(30.0), //+-
            accel_speed: 6.0,
            brake_speed: 3.0,
            wheel_base: 50.0,
            track: 30.0
        }
    }
}

fn main(){
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(SpeedTimer(Timer::from_seconds(0.5, TimerMode::Repeating)))
        .insert_resource(Time::<Fixed>::from_hz(120.0))
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                update_car_location
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
){
    commands.spawn((
        Camera2d,
        Camera {
            hdr: true, // 1. HDR is required for bloom
            clear_color: ClearColorConfig::Custom(Color::WHITE),
            ..default()
        },
    ));
    let car = Car{
        max_mov_speed: 150.0,
        accel_speed: 40.0,
        brake_speed: 60.0,
        ..Default::default()
    };

    let car_shape = Rectangle::new(car.track, car.wheel_base);
    let car_color = Color::srgba(1.0, 0.0,0.0, 0.3);

    let wheel_base = car.wheel_base;
    let max_steering = car.max_steering_rad;
    let turning_radius = calculate_turning_radius(&car);

    commands.spawn(
        (
            Mesh2d(meshes.add(car_shape)),
            MeshMaterial2d(materials.add(car_color)),
            Transform::from_translation(Vec3::new(0., 0., 0.)),
            car
        )
    ).with_children(|parent| {
        let wheel_size = Rectangle::new(6.0, 12.0); // width x height
        let wheel_color = Color::srgba(0.1, 0.1, 0.1, 1.0); // dark wheels

        let wheel_mesh = meshes.add(wheel_size);
        let wheel_material = materials.add(wheel_color);

        // Wheel positions relative to the car body
        let positions = [
            Vec3::new(-12.0, 20.0, 1.0), // front-left
            Vec3::new(12.0, 20.0, 1.0),  // front-right
            Vec3::new(-12.0, -20.0, 1.0), // rear-left
            Vec3::new(12.0, -20.0, 1.0),  // rear-right
        ];

        for pos in positions {
            parent.spawn((
                Mesh2d(wheel_mesh.clone()),
                MeshMaterial2d(wheel_material.clone()),
                Transform::from_translation(pos),
            ));

        }

        //Draw steering angle
        let arrow = Rectangle::new(2.0, 60.0);
        let arrow_color = Color::srgba(0.1, 0.1, 0.1, 1.0);
        let arrow_mesh = meshes.add(arrow);
        let arrow_material = materials.add(arrow_color);
        // parent.spawn(
        //     (
        //         Mesh2d(arrow_mesh.clone()),
        //         MeshMaterial2d(arrow_material.clone()),
        //         Transform::from_translation(Vec3::new(0.0, 20.0, 0.0)),
        //         )
        // );

        let mut max_steering_angle_transform = Transform::from_translation(Vec3::new(0.0, 20.0, 0.0));
        max_steering_angle_transform.rotation = Quat::from_axis_angle(Vec3::Z, max_steering);
        parent.spawn(
            (
                Mesh2d(arrow_mesh.clone()),
                MeshMaterial2d(arrow_material.clone()),
                max_steering_angle_transform
            )
        );

        //Draw Turning Radius
        let circle = Circle::new(turning_radius);
        let circle_color = Color::srgba(0.0, 0.0, 1.0, 1.0);
        let circle_mesh = meshes.add(circle);
        let circle_material = materials.add(circle_color);
        parent.spawn(
            (
                Mesh2d(circle_mesh),
                MeshMaterial2d(circle_material),
                Transform::from_translation(Vec3::new(turning_radius, -wheel_base/2.0, 0.0)),
            )
        );

    });



}

fn update_car_location(
    query: Single<(&mut Car, &mut Transform, &mut Children)>,
    mut child_query: Query<(&mut Transform), Without<Car>>,
    keycode: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut timer: ResMut<SpeedTimer>
){

    let (mut car, mut transform, children) = query.into_inner();
    let [mut car_head_transform_left, mut car_head_transform_right] =
        child_query.get_many_mut([children[0], children[1]]).unwrap();

    
    if keycode.pressed(KeyCode::ArrowUp) {
        car.mov_speed = clamp(
            car.mov_speed + car.accel_speed * time.delta_secs(),
            -car.max_mov_speed,
            car.max_mov_speed,
        );
    } else if keycode.pressed(KeyCode::ArrowDown) {
        car.mov_speed = clamp(
            car.mov_speed - car.brake_speed * time.delta_secs(),
            -car.max_mov_speed,
            car.max_mov_speed,
        );
    } else {
        if car.mov_speed > 0.0 {
            car.mov_speed = clamp(
                car.mov_speed - car.accel_speed * time.delta_secs(),
                0.0,
                car.max_mov_speed,
            );
        } else {
            car.mov_speed = clamp(
                car.mov_speed + car.accel_speed * time.delta_secs(),
                -car.max_mov_speed,
                0.0,
            );
        }
    }
    
    if keycode.pressed(KeyCode::ArrowLeft) {
        car.steering_rad = clamp(
            car.steering_rad + f32::to_radians(60.0) * time.delta_secs(),
            -car.max_steering_rad,
            car.max_steering_rad,
        );
    } else if keycode.pressed(KeyCode::ArrowRight) {
        car.steering_rad = clamp(
            car.steering_rad - f32::to_radians(60.0) * time.delta_secs(),
            -car.max_steering_rad,
            car.max_steering_rad,
        );
    } else {
        car.steering_rad = car.steering_rad * 0.9;
    }

    let beta = f32::atan(0.5 * f32::tan(car.steering_rad));
    let omega = (car.mov_speed / car.wheel_base) * f32::cos(beta) * f32::tan(car.steering_rad) * time.delta_secs();
    transform.rotate_z(omega);

    let forward_dir = transform.rotation * Quat::from_rotation_z(beta) * Vec3::Y;

    transform.translation += forward_dir * car.mov_speed * time.delta_secs();
    
    car_head_transform_left.rotation = Quat::from_rotation_z(car.steering_rad);
    car_head_transform_right.rotation = Quat::from_rotation_z(car.steering_rad);
    
    let extents = Vec3::from((SIZE / 2.0, 0.0));
    transform.translation = transform.translation.min(extents).max(-extents);
    
    if timer.0.tick(time.delta()).just_finished() {
        println!("Speed: {:.2}", car.mov_speed);
        println!("Steering Angle (deg): {:.2}", car.steering_rad.to_degrees());
    }
}

pub fn calculate_turning_radius(car: &Car) -> f32{
    let beta = f32::atan(0.5 * f32::tan(car.max_steering_rad));
    car.wheel_base / (f32::tan(car.max_steering_rad) * f32::cos(beta))
}

#[inline]
pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.min(max).max(min)
}