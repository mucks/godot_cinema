extends KinematicBody


# Declare member variables here. Examples:
# var a = 2
# var b = "text"

var camera_angle = 0
var mouse_sensitivity = 0.1
var camera_change = Vector2()

var velocity = Vector3()
var direction = Vector3()

const FLY_SPEED = 40
const FLY_ACCEL = 4

var gravity = -9.8 * 3
var speed = 0
const MAX_SPEED = 20
const MAX_RUNNING_SPEED = 30
const ACCELL = 2
const DEACCELL = 6


# jumping
var jump_height = 15

# Called when the node enters the scene tree for the first time.
func _ready():
	pass # Replace with function body.

func _physics_process(delta):
	aim()
	walk(delta)

func _input(event):
	if event is InputEventMouseMotion:
		camera_change = event.relative

func walk(delta):
	direction = Vector3()

	# get the rotation of the Camera
	var aim = $Head/Camera.get_global_transform().basis

	# check input and change direction
	if Input.is_action_pressed("move_forward"):
		direction -= aim.z
	if Input.is_action_pressed("move_backward"):
		direction += aim.z
	if Input.is_action_pressed("move_left"):
		direction -= aim.x
	if Input.is_action_pressed("move_right"):
		direction += aim.x

	direction = direction.normalized()

	velocity.y += gravity * delta

	var temp_velocity = velocity
	temp_velocity.y = 0

	if Input.is_action_pressed("move_sprint"):
		speed = MAX_RUNNING_SPEED
	else:
		speed = MAX_SPEED

	#where would the player go at max speed
	var target = direction * speed

	var acceleration
	if direction.dot(temp_velocity) > 0:
		acceleration = ACCELL
	else:
		acceleration = DEACCELL



    # calculate a portion of distance to go
	temp_velocity = temp_velocity.linear_interpolate(target, acceleration * delta)

	velocity.x = temp_velocity.x
	velocity.z = temp_velocity.z

	velocity = move_and_slide(velocity, Vector3(0, 1, 0))

	#jump
	if is_on_floor() and Input.is_action_just_pressed("jump"):
		velocity.y = jump_height
	 
	# move
	move_and_slide(velocity)

func fly(delta):
	direction = Vector3()

	# get the rotation of the Camera
	var aim = $Head/Camera.get_global_transform().basis

	# check input and change direction
	if Input.is_action_pressed("move_forward"):
		direction -= aim.z
	if Input.is_action_pressed("move_backward"):
		direction += aim.z
	if Input.is_action_pressed("move_left"):
		direction -= aim.x
	if Input.is_action_pressed("move_right"):
		direction += aim.x

	direction = direction.normalized()
	
	var target = direction * FLY_SPEED

    # calculate a portion of distance to go
	velocity = velocity.linear_interpolate(target, FLY_ACCEL * delta) 
	 
	# move
	move_and_slide(velocity)

func aim():
	if camera_change.length() > 0:
		$Head.rotate_y(deg2rad(-camera_change.x * mouse_sensitivity))
		var change = -camera_change.y * mouse_sensitivity
		if change + camera_angle < 90 and change + camera_angle > -90:
			$Head/Camera.rotate_x(deg2rad(change))
			camera_angle += change
		camera_change = Vector2()





# Called every frame. 'delta' is the elapsed time since the previous frame.
#func _process(delta):
#	pass
