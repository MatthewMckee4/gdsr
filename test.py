import gdstk

import gdsr

gdsr_library = gdsr.Library("test")

gdsr_cell = gdsr.Cell("test")

gdsr_cell.add(gdsr.Polygon([(0, 0), (1, 0), (1, 1), (0, 1)]))

gdsr_library.add(gdsr_cell)

gdsr_library.to_gds("test.gds", 1e-6, 1e-9)


gdstk_library = gdstk.Library("test", 1e-6, 1e-9)

gdstk_cell = gdstk.Cell("test")

gdstk_cell.add(gdstk.Polygon([(0, 0), (1, 0), (1, 1), (0, 1)]))

gdstk_library.add(gdstk_cell)

gdstk_library.write_gds("test1.gds")
