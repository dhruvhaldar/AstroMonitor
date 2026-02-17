## 2024-05-22 - Filtered Lists in Egui
**Learning:** When implementing client-side filtering in immediate mode GUI (egui), mapping filtered indices back to the original data structure allows for efficient virtualization without duplicating the data.
**Action:** Use a transient index map (Vec<usize>) in the render loop for filtered views.
