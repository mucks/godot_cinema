extends LineEdit


# Declare member variables here. Examples:
# var a = 2
# var b = "text"

var is_focused = false

# Called when the node enters the scene tree for the first time.
func _ready():
	pass # Replace with function body.


func _process(_delta):
	if Input.is_action_just_pressed("open_menu"):
		if is_focused:
			hide()
			release_focus()
			is_focused = false
		else:
			show()
			grab_focus()
			is_focused = true

func _on_YoutubeUrlEntry_text_entered(_txt):
	hide()
	clear()
	release_focus()




# Called every frame. 'delta' is the elapsed time since the previous frame.
#func _process(delta):
#	pass
