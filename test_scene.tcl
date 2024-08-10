# Tcl may actually be a pretty great format for setting up a scene
# The downside would be that it is probably not great at being generated
# by an editor

light point 0.3 [pos 10.0 0.0 10.0]
light ambient [color 0.1 0.1 0.2]
model "Model name" [pos 0.0 0.0 0.0] [rot 0.0 0.0 0.0]
set villian [model "Villian.file" [pos 10 0 0] [rot 0 0 0]]

# That is actually pretty clean 
# we could also include scripts fairly cleanly
script {
	wait [goto $villian [pos 0 0 0]]
	say $villian "I am evil, prepare to die"
}

# This would kind of insinuate that loading a scene is just clearing the
# world and then loading one of these scripts. Potentially small scene changes
# could be done by just loading an additional scene file without clearing the
# existing world
script {
	wait [is_dead $villian]
	load_file "happy_scene.tcl"
}

# You wouldn't want to use these scripts for implementing any game features
# but for scripting content I think this could be pretty great
