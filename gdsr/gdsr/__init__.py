import logging
import sys
from typing import List, Optional

from gdsr.dependencies import check_module

from .. import _gdsr

if sys.version_info >= (3, 11):
    from typing import Self
else:
    from typing_extensions import Self


class Cell(_gdsr.Cell): ...


class CellReference(_gdsr.CellReference): ...


class ElementReference(_gdsr.ElementReference): ...


class Grid(_gdsr.Grid): ...


from .._gdsr import HorizontalPresentation  # type: ignore


class Library(_gdsr.Library): ...


class Node(_gdsr.Node): ...


class Path(_gdsr.Path): ...


class Point(_gdsr.Point): ...


class PointIterator(_gdsr.PointIterator): ...


class Polygon(_gdsr.Polygon):
    def visualise(self):
        """Visualise the polygon using the matplotlib module.

        :return matplotlib.figure.Figure | None: The figure containing the visualisation
        If the matplotlib module is not available, None is returned.
        """
        if not check_module("matplotlib"):
            return logging.info("The matplotlib module is not available.")

        import matplotlib.pyplot as plt

        figure = plt.figure()  # type: ignore

        x = [point.x for point in self.points]
        y = [point.y for point in self.points]

        x.append(x[0])
        y.append(y[0])

        plt.plot(x, y, marker="o")  # type: ignore
        plt.fill(x, y, alpha=0.3)  # type: ignore

        (min_x, min_y), (max_x, max_y) = self.bounding_box

        plt.xlim(min_x - 1, max_x + 1)  # type: ignore
        plt.ylim(min_y - 1, max_y + 1)  # type: ignore
        plt.axhline(0, color="grey", lw=0.5)  # type: ignore
        plt.axvline(0, color="grey", lw=0.5)  # type: ignore
        plt.grid()  # type: ignore
        plt.show()  # type: ignore

        return figure

    @staticmethod
    def create_interactive_polygon():
        if not check_module("matplotlib"):
            return logging.info("The matplotlib module is not available.")

        import matplotlib.pyplot as plt
        from matplotlib.widgets import Button

        """Interactively create a polygon using mouse clicks and return the polygon."""
        points: List[Point] = []

        def onclick(event) -> None:
            if event.inaxes is not ax:
                return
            points.append(Point(event.xdata, event.ydata))
            ax.plot(event.xdata, event.ydata, "ro")  # Mark the point
            if len(points) > 1:
                x = [point.x for point in points]
                y = [point.y for point in points]
                ax.plot(x, y, "b-")  # Connect the points
            fig.canvas.draw()

        def finalize(event) -> None:
            if len(points) < 3:
                print("At least 3 points are required to form a polygon.")
                return
            polygon = Polygon(points)  # Create a Polygon instance
            polygon.visualise()  # Visualize the created polygon
            plt.close(fig)  # Close the interactive window

        # Set up the figure and axes
        fig, ax = plt.subplots()
        ax.set_xlim(-10, 10)
        ax.set_ylim(-10, 10)
        ax.axhline(0, color="grey", lw=0.5)
        ax.axvline(0, color="grey", lw=0.5)
        ax.grid()

        # Connect the click event
        cid = fig.canvas.mpl_connect("button_press_event", onclick)

        # Button to finalize the polygon
        button_ax = fig.add_axes([0.8, 0.01, 0.1, 0.05])
        button = Button(button_ax, "Finalize")
        button.on_clicked(finalize)

        plt.show()

        # Return the created polygon after finalization
        return Polygon(points) if len(points) >= 3 else None


class Text(_gdsr.Text):
    def visualise(self):
        """Visualise the text using the matplotlib module.

        :return matplotlib.figure.Figure | None: The figure containing the visualisation
        .If the matplotlib module is not available, None is returned.
        """
        if not check_module("matplotlib"):
            return logging.info("The matplotlib module is not available.")

        import matplotlib.pyplot as plt

        figure = plt.figure()  # type: ignore

        plt.text(  # type: ignore
            self.origin.x,
            self.origin.y,
            self.text,
            fontsize=12 * self.magnification,
            rotation=self.angle,
            ha=self.horizontal_presentation.name.lower()
            if self.horizontal_presentation != HorizontalPresentation.Centre
            else "center",
            va=self.vertical_presentation.name.lower()
            if self.vertical_presentation != VerticalPresentation.Middle
            else "center",
        )

        plt.xlim(-10, 10)  # type: ignore
        plt.ylim(-10, 10)  # type: ignore
        plt.axhline(0, color="grey", lw=0.5)  # type: ignore
        plt.axvline(0, color="grey", lw=0.5)  # type: ignore
        plt.grid()  # type: ignore
        plt.show()  # type: ignore

        return figure


from .._gdsr import VerticalPresentation  # type: ignore

__all__ = [
    "Cell",
    "CellReference",
    "ElementReference",
    "Grid",
    "HorizontalPresentation",
    "Library",
    "Node",
    "Path",
    "Point",
    "PointIterator",
    "Polygon",
    "Text",
    "VerticalPresentation",
]
