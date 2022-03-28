# Lab 4: Profiling and improving the performance of a program

# Summary

In this assignment, we were given a working program that simulates a hackathon, using three different types of threads: Students, Idea Generators, and Package Downloaders. As command line arguments, the number of students, ideas, idea generators, packages, and package downloaders are all configurable. Using flamegraph-based profiling and benchmarking using hyperfine, I was able to incrementally improve the performance of the program through a multitude of changes, eventually resulting in a significant speedup of the program (across multiple different sizes of input arguments). The all testing and benchmarking was done on the ecetesla0 server.

# Technical details

There were 5 main changes that were made to improve the performance of the program. All these changes were motivated by observations of the flamegraph. In general, whenever a large portion of the flamegraph was being taken up by one function call, it signified that lots of time was being spent performing that function call and that there was room for improvement in that area.

**File reading in package downloader:** In the original [flamegraph](../benchmarks/starter/flamegraph.svg), a significant amount of time could be seen spent in PackageDownloader::run(), specifically reading the packages text file and iterating/indexing through the lines. The reason why so much time was spent doing this was because every single time an individual package was being read and "downloaded", the file was being read and iterated through. This was extremely repetitive and inefficient. To fix this, I simply moved the file reading code into main to be only done once, and then split up the resulting lines vector into equal sections to be passed to each package downloader thread.

**Cross product and file reading in idea generator:** Next, it was noticed that in the flamegraph an abnormally large portion of time was being spent in IdeaGenerator::run() when trying to get the next idea name. This was because the products and consumers text files were being read, and more significantly, the cross product was being computed in each call of get_next_idea_name(). Similar to the file reading in package downloader, these calls are very repetitive and don't change. Once again, this code can be moved out into main and be called only once. The resulting vector of the cross product is then wrapped in an atomic reference count and then passed to each idea generator thread to be accessed.

**Mutex Aquisition Optimizations:** In the next iteration of the flamegraph,

# Testing for correctness

Testing for correctness in this assignment is relatively straightforward. After each change I made, I ensured that the program was still correct by observing the final checksums outputted. Firstly, the final idea generator checksum should equal to the final student idea checksum, and the final package downloader checksum is equal to the final student package checksum. This ensures that the student threads have created the right number of ideas using the right packages.

Afterwards, the final checksums were compared to the final checksums outputted by the starter code (when run with the same input arguments). As long as the checksums still match, we know that the same ideas are being generated as the intended behaviour, using the same packages. This means that the output of the hackathon simulation has not changed and is therefore correct.

# Testing for performance.

Something about how you tested this code for performance and know it is faster (about 3 paragraphs, referring to the flamegraph as appropriate.)

As alluded to earlier, each change that was made in an attempt to improve the performance of the program was done so based on some observation of the flamegraph. After each change, hyperfine was used to benchmark/time the resulting code, validating that the change did in fact speed up the program.

In general, my workflow was as such (each iteration):
- Profile the program using the flamegraph
- Identify an area that can be improved
- Optimize program to improve that area
- Benchmark program to validate improvement (program should be faster now)

Essentially, iteratively benchmarking the changes I made using hyperfine and ensuring that each change made an improvement in runtime was how I tested for performance.

*An important note:* In order to not bias the improvements made to only one set of input sizes, both profiling and benchmarking was done on multiple  sets of input arguments during each improvement iteration. I usually used

Reference the flamegraphs and benchmarks....
