# Approach

* Convexify faces [skip for now]
  * can use Hertel-Mehlhorn to get convex decomposition from triangulation
  * https://doc.cgal.org/Manual/3.2/doc_html/cgal_manual/Partition_2/Chapter_main.html
* Orient faces [done]
  * Associate a single direct isometry to each face [done]
  * Fail if isometries are inconsistent [done]
* For every pair of faces
  * Use bounding boxes to eliminate far pairs
    * Bounding boxes should have lower bounds on the dimensions so that we can't miss parallel close-to-planar pairs
    * Also use close pairs of faces to determine close pairs of creases and crease-face pairs
  * Also eliminate pairs joined by creases since these will be handled separately [done]
  * Determine if faces are parallel [done]
    * Check if they overlap in planar projection after slightly shrunk [done]
    * Compute and save intersection for later [done]
  * If not, check if slightly-shrunk versions of faces intersect [done]
    * Fail if they do [done]
* Check face-crease incidences
  * (original) crease is close to plane, and shrunk projected crease intersects shrunk face
* Check crease-crease incidences
  * Creases are close to collinear, and shrunk projected creases intersect
* Find triangles in intersection graph of faces
  * For each triangle, check if there is a triple intersection of slightly-shrunk faces
* Generate constraints


# Todo

* Test on crease patterns
  * Twist
  * Waterbomb [bunny]
  * Traditional models
    * Crane [with extended wings]
    * Frog
  * Mooser's train [in parts]
    * http://creatingorigami.com/collateral/pdfs/Emmanuel%20Mooser%20-%20Mooser%27s%20Train.pdf
  * Look through traditional/modern collections and books like Origami Design Secrets for more ideas
* Would be nice to implement "canonicalization" of FOLD files in js
