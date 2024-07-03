# Class Diagrams

### Class diagrams for gdsr

```mermaid
classDiagram
    class Cell {
      +List~ArrayReference~ arrayReferences
      +List~Boundary~ polygons
      +List~Box~ boxes
      +List~Node~ nodes
      +List~Path~ paths
      +List~Reference~ references
      +List~Text~ texts
    }

    class ArrayReference
    class Polygon
    class Box
    class Node
    class Path
    class Reference
    class Text

    Cell "1" *-- "many" ArrayReference : contains
    Cell "1" *-- "many" Polygon : contains
    Cell "1" *-- "many" Box : contains
    Cell "1" *-- "many" Node : contains
    Cell "1" *-- "many" Path : contains
    Cell "1" *-- "many" Reference : contains
    Cell "1" *-- "many" Text : contains
```