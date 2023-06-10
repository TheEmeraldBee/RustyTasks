# RustyTasks
Your own terminal based to-do list with folders to store your tasks

This is a terminal tool that will help you keep track of what you need to do without using yet another free website!

Rusty Tasks uses a pretty table style renderer that makes your tasks look like this!

```
+-------------+-----------------------------------------------+
| Folder      | Tasks                                         |
+=============+===============================================+
| Cool Things |  ID | Task                     | Status       |
|             | ====+==========================+============= |
|             |  63 | This is a test task that | In Progress  |
|             |     | will automatically add   |              |
|             |     | newlines                 |              |
+-------------+-----------------------------------------------+
```

## Installation
First, make sure that you have sqlite installed, if you don't, install it with your favorite package manager.

Then, installing RustyTasks is as simple as having cargo and running ```cargo install rusty_tasks```

## Usage
To start, run Rusty Tasks by typing ```rt``` in your terminal! It will show you a list of all of the
awesome things you can do with it! But we'll explain a couple of them here!

### List
This one is simple, type ```rt list``` or ```rt list -f "folder_name_here"``` To see all of your tasks, or just the tasks under the specific folder!

### Add

This one is how you add a new task!

Using ```rt add``` followed by a ```-f "folder_name_here"``` and ```-t "task_here"``` You will have created a task, and it will show you everything about your new task!

### Update

This one is the most complicated,

Type ```rt update``` followed with ```-i id_here``` replacing id_here with the unique id of the task, shown in the example at the top, you can add a ton of flags to customize your ticket!

Examples

```
//           ID     Change the folder         Change the task     Set the status
rt update -i 123 -f "New Folder Location" -t "Change Task Name" -s incomplete
```
That is a simple example, but it will do what you want.

**You Can However** Set Custom Statuses!

Using the ```-c``` flag, you can set the status of your task to whatever you want!

Example
```rt update -i 123 -c "Needs confirmation" ```
This will make sure you can know what is going on with and why whatever task isn't done!

### Delete
This one is also simple. Get the unique id of the task, and type ```rt delete -i id_here``` to pop up a confirmation screen.
If that looks like the ticket you were wanting to delete, type ```Y``` or ```YES``` to confirm the deletion. If it isn't the right ticket, type any other character to cancel!

# Roadmap

Although things work pretty well, we have a couple of goals before 1.0 is ready!

- [ ] Add a way to bind a set of tasks to an actual folder on your system (Maybe initialize a .db file in that location, and access with a ```--current_location``` flag?)
- [ ] Allow the use of helix/vim to create and modify tasks
- [ ] Add some more prettyness and customization options to the way it looks
- [ ] Add a subfolder system and a config option to desplay the tasks in a small amount of depth, all the way to max depth!
