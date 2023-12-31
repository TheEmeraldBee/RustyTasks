# Rusty Tasks V 0.2.0
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
This one is simple, type ```rt list``` or ```rt list -f "folder/name/here"``` To see all of your tasks, or just the tasks under the specific folder!

Optional: You can use the ```-d``` flag followed by a number to set the max depth that the system will recurse into folders (Default of 3)

### Add

This one is how you add a new task!

Using ```rt add``` followed by a ```-f "folder/name/here"``` and ```-t "task_here"``` You will have created a task, and it will show you everything about your new task!

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

### Directories
This one is cool! Any paths with a ```/``` character think of the path that follows as a subdirectory!
Ex: ```hello/world```
This command would be thought of as a subdirectory hello and another subdirectory world, allowing infinite customization.

In order to list these, simple type it as the same way you created it!

**However** You can only see 5 levels into the tasks, if you want to see deeper, list a subdirectory, and you will be able to see 5 levels deep into that directory!

# Roadmap

Although things work pretty well, we have a couple of goals before 1.0 is ready!

- [ ] Allow the use of helix/vim to create and modify tasks
- [ ] Add some more prettyness and customization options to the way it looks
- [x] Add a subfolder system that allows for infinite recursion!
