pub mod app;
pub mod instance;
pub mod mesh;
pub mod renderer;
pub mod shader;
pub mod textures;
pub mod vertex;

/*
   Object Rendering:
       1. Set of vertices and indices created for a mesh, referenced by object
       2. For each frame:
           a. Scale, Rotation, Transformation turned into 4x4 transform matrix (object.instance())
           b. Copied to instance storage in the CPU at the correct location (update_instances())
           c. Entire instance buffer is copied to VRAM (update_gpu())
           d. Each mesh's instances gets own draw_instanced call


*/
