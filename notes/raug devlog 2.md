- I'm a couple more weeks and several thousand more lines of code deep into development of Raug
- What's the shiny new stuff?
	- FFT module
	- Serde support
	- VST3 Plugin and GUI!
- What have I learned?
	- It isn't necessary to copy Max and Pure Data's designs verbatim
		- They are old and originally designed for much slower computers
		- They separate audio and control-rate signals mostly for legacy performance reasons (from what I understand)
		- This kind of design is harder to implement and work with
		- Raug has always been audio-rate-only, but this clarified some things
			- Gone are Messages
				- I was a little confused about what I was trying to do here
			- New Signal trait / AnySignal enum
	- Computers are fast when you let them be
		- Designing structures around cache/memory efficiency and vectorization will go a LONG way
		- Vec of Enums versus Enum of Vecs
			- Similar to "Struct of Arrays versus Array of Structs" in game engine ECS design
		- Take advantage of niche properties of your design and goals
			- Factor out large structures into smaller ones that can work independently
