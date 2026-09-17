"""Integration tests for pyratatui.

These tests validate the Python API surface without opening a real terminal.
Terminal rendering itself is covered by the pty-based tests in
test_inline_viewport.py.
"""

from __future__ import annotations

import inspect
from datetime import date as _pydate

import pytest


class TestColor:
    def test_named_colors(self) -> None:
        from pyratatui import Color

        assert Color.red() is not None
        assert Color.green() is not None
        assert Color.blue() is not None
        assert Color.cyan() is not None
        assert Color.reset() is not None

    def test_indexed_color(self) -> None:
        from pyratatui import Color

        c = Color.indexed(196)
        assert c is not None
        assert "196" in repr(c)

    def test_rgb_color(self) -> None:
        from pyratatui import Color

        c = Color.rgb(255, 128, 0)
        assert c is not None

    def test_equality(self) -> None:
        from pyratatui import Color

        assert Color.red() == Color.red()
        assert Color.red() != Color.green()


class TestModifier:
    def test_bold(self) -> None:
        from pyratatui import Modifier

        assert Modifier.bold() is not None

    def test_or_combine(self) -> None:
        from pyratatui import Modifier

        m = Modifier.bold() | Modifier.italic()
        assert m is not None


class TestStyle:
    def test_default_style(self) -> None:
        from pyratatui import Style

        s = Style()
        assert s.foreground is None
        assert s.background is None

    def test_fg_bg_chain(self) -> None:
        from pyratatui import Color, Style

        s = Style().fg(Color.red()).bg(Color.black())
        assert s.foreground == Color.red()
        assert s.background == Color.black()

    def test_modifier_chain(self) -> None:
        from pyratatui import Style

        s = Style().bold().italic().underlined()
        assert s is not None

    def test_patch(self) -> None:
        from pyratatui import Color, Style

        base = Style().fg(Color.red())
        overlay = Style().bg(Color.black())
        merged = base.patch(overlay)
        assert merged.foreground == Color.red()
        assert merged.background == Color.black()

    def test_repr(self) -> None:
        from pyratatui import Style

        assert "Style" in repr(Style())


class TestSpan:
    def test_plain_span(self) -> None:
        from pyratatui import Span

        s = Span("hello")
        assert s.content == "hello"
        assert s.style is None
        assert s.width() == 5

    def test_styled_span(self) -> None:
        from pyratatui import Color, Span, Style

        s = Span("hi", Style().fg(Color.green()))
        assert s.style is not None

    def test_styled_method(self) -> None:
        from pyratatui import Color, Span, Style

        s = Span("test").styled(Style().fg(Color.blue()))
        assert s.style is not None


class TestLine:
    def test_from_string(self) -> None:
        from pyratatui import Line

        ln = Line.from_string("Hello World")
        assert len(ln.spans) == 1
        assert ln.width() == 11

    def test_alignment(self) -> None:
        from pyratatui import Line

        ln = Line.from_string("test")
        assert ln.centered() is not None
        assert ln.right_aligned() is not None
        assert ln.left_aligned() is not None

    def test_push_span(self) -> None:
        from pyratatui import Line, Span

        ln = Line()
        ln.push_span(Span("a"))
        ln.push_span(Span("b"))
        assert len(ln.spans) == 2


class TestText:
    def test_from_string_multiline(self) -> None:
        from pyratatui import Text

        t = Text.from_string("line1\nline2\nline3")
        assert t.height == 3

    def test_push_str(self) -> None:
        from pyratatui import Text

        t = Text()
        t.push_str("first")
        t.push_str("second")
        assert t.height == 2

    def test_centered(self) -> None:
        from pyratatui import Text

        t = Text.from_string("hello").centered()
        assert t is not None


class TestRect:
    def test_creation(self) -> None:
        from pyratatui import Rect

        r = Rect(0, 0, 80, 24)
        assert r.x == 0
        assert r.y == 0
        assert r.width == 80
        assert r.height == 24

    def test_area(self) -> None:
        from pyratatui import Rect

        assert Rect(0, 0, 80, 24).area() == 1920

    def test_inner(self) -> None:
        from pyratatui import Rect

        r = Rect(0, 0, 80, 24)
        inner = r.inner(2, 1)
        assert inner.width == 76
        assert inner.height == 22

    def test_edges(self) -> None:
        from pyratatui import Rect

        r = Rect(10, 5, 20, 10)
        assert r.right == 30
        assert r.bottom == 15
        assert r.left == 10
        assert r.top == 5

    def test_is_empty(self) -> None:
        from pyratatui import Rect

        assert Rect(0, 0, 0, 0).is_empty()
        assert not Rect(0, 0, 1, 1).is_empty()

    def test_intersection(self) -> None:
        from pyratatui import Rect

        a = Rect(0, 0, 10, 10)
        b = Rect(5, 5, 10, 10)
        i = a.intersection(b)
        assert i is not None
        assert i.width == 5
        assert i.height == 5

    def test_no_intersection(self) -> None:
        from pyratatui import Rect

        a = Rect(0, 0, 5, 5)
        b = Rect(10, 10, 5, 5)
        assert a.intersection(b) is None

    def test_equality(self) -> None:
        from pyratatui import Rect

        assert Rect(0, 0, 80, 24) == Rect(0, 0, 80, 24)
        assert Rect(0, 0, 80, 24) != Rect(1, 0, 80, 24)


class TestConstraint:
    def test_all_variants(self) -> None:
        from pyratatui import Constraint

        assert Constraint.length(10) is not None
        assert Constraint.percentage(50) is not None
        assert Constraint.fill(1) is not None
        assert Constraint.min(5) is not None
        assert Constraint.max(20) is not None
        assert Constraint.ratio(1, 3) is not None


class TestLayout:
    def test_vertical_split(self) -> None:
        from pyratatui import Constraint, Direction, Layout, Rect

        area = Rect(0, 0, 80, 24)
        chunks = (
            Layout()
            .direction(Direction.Vertical)
            .constraints([Constraint.length(3), Constraint.fill(1)])
            .split(area)
        )
        assert len(chunks) == 2
        assert chunks[0].height == 3
        assert chunks[1].y == 3

    def test_horizontal_split(self) -> None:
        from pyratatui import Constraint, Direction, Layout, Rect

        area = Rect(0, 0, 80, 24)
        chunks = (
            Layout()
            .direction(Direction.Horizontal)
            .constraints([Constraint.percentage(50), Constraint.percentage(50)])
            .split(area)
        )
        assert len(chunks) == 2

    def test_nested_layout(self) -> None:
        from pyratatui import Constraint, Direction, Layout, Rect

        area = Rect(0, 0, 100, 40)
        outer = (
            Layout()
            .direction(Direction.Vertical)
            .constraints([Constraint.length(5), Constraint.fill(1)])
            .split(area)
        )
        inner = (
            Layout()
            .direction(Direction.Horizontal)
            .constraints([Constraint.fill(1), Constraint.fill(1)])
            .split(outer[1])
        )
        assert len(inner) == 2


class TestBlock:
    def test_default(self) -> None:
        from pyratatui import Block

        assert Block() is not None

    def test_chain(self) -> None:
        from pyratatui import Block, BorderType, Color, Style

        b = (
            Block()
            .title("Test")
            .bordered()
            .border_type(BorderType.Rounded)
            .style(Style().fg(Color.cyan()))
            .padding(1, 1, 0, 0)
        )
        assert "Test" in repr(b)

    def test_borders(self) -> None:
        from pyratatui import Block

        b = Block().borders(top=True, right=False, bottom=True, left=False)
        assert b is not None


class TestParagraph:
    def test_from_string(self) -> None:
        from pyratatui import Paragraph

        assert Paragraph.from_string("Hello") is not None

    def test_chain(self) -> None:
        from pyratatui import Block, Color, Paragraph, Style

        p = (
            Paragraph.from_string("Test")
            .block(Block().bordered())
            .style(Style().fg(Color.white()))
            .wrap(True, True)
            .scroll(2, 0)
            .centered()
        )
        assert p is not None


class TestList:
    def test_list_creation(self) -> None:
        from pyratatui import List, ListItem

        items = [ListItem(f"Item {i}") for i in range(5)]
        lst = List(items)
        assert lst is not None
        assert "5" in repr(lst)

    def test_list_state(self) -> None:
        from pyratatui import ListState

        state = ListState()
        state.select(2)
        assert state.selected == 2
        state.select_next()
        assert state.selected == 3
        state.select_previous()
        assert state.selected == 2
        state.select(None)
        assert state.selected is None

    def test_list_chain(self) -> None:
        from pyratatui import Block, Color, List, ListItem, Style

        lst = (
            List([ListItem("item")])
            .block(Block().bordered())
            .highlight_style(Style().fg(Color.yellow()))
            .highlight_symbol("\u25b6 ")
        )
        assert lst is not None


class TestTable:
    def test_table_state(self) -> None:
        from pyratatui import TableState

        s = TableState()
        s.select(0)
        assert s.selected == 0
        s.select_next()
        assert s.selected == 1

    def test_constructor_kwargs(self) -> None:
        from pyratatui import Constraint, Row, Table

        header = Row.from_strings(["Name", "Value"])
        table = Table(
            [Row.from_strings(["Alice", "42"])],
            column_widths=[Constraint.length(20), Constraint.fill(1)],
            header=header,
        )
        assert table is not None
        assert "Table" in repr(table)

    def test_constructor_defaults(self) -> None:
        from pyratatui import Row, Table

        table = Table([Row.from_strings(["a"])])
        assert table is not None


class TestGauge:
    def test_gauge(self) -> None:
        from pyratatui import Color, Gauge, Style

        g = Gauge().percent(75).style(Style().fg(Color.green())).label("75%")
        assert "75" in repr(g)

    def test_gauge_ratio(self) -> None:
        from pyratatui import Gauge

        assert Gauge().ratio(0.5) is not None

    def test_line_gauge(self) -> None:
        from pyratatui import LineGauge

        lg = LineGauge().ratio(0.65).line_set("double")
        assert "0.65" in repr(lg)


class TestBarChart:
    def test_bar_chart(self) -> None:
        from pyratatui import Bar, BarChart, BarGroup

        chart = BarChart().data(BarGroup([Bar(10, "Jan"), Bar(20, "Feb")])).bar_width(5).max(30)
        assert chart is not None

    def test_bar_repr(self) -> None:
        from pyratatui import Bar

        assert "42" in repr(Bar(42, "Test"))


class TestSparkline:
    def test_sparkline(self) -> None:
        from pyratatui import Color, Sparkline, Style

        s = Sparkline().data([10, 20, 15, 35, 25]).style(Style().fg(Color.green()))
        assert "5" in repr(s)


class TestScrollbar:
    def test_scrollbar_state(self) -> None:
        from pyratatui import ScrollbarState

        assert ScrollbarState().content_length(100).position(20) is not None

    def test_scrollbar(self) -> None:
        from pyratatui import Scrollbar, ScrollbarOrientation

        assert Scrollbar(ScrollbarOrientation.VerticalRight) is not None


class TestTabs:
    def test_tabs(self) -> None:
        from pyratatui import Color, Style, Tabs

        tabs = (
            Tabs(["Tab 1", "Tab 2", "Tab 3"])
            .select(1)
            .highlight_style(Style().fg(Color.yellow()))
            .divider(" | ")
        )
        assert "3" in repr(tabs)
        assert "selected=1" in repr(tabs)


class TestCalendarDate:
    def test_today(self) -> None:
        from pyratatui import CalendarDate

        d = CalendarDate.today()
        today = _pydate.today()
        assert d.year == today.year
        assert d.month == today.month
        assert d.day == today.day

    def test_from_ymd_valid(self) -> None:
        from pyratatui import CalendarDate

        d = CalendarDate.from_ymd(2024, 3, 15)
        assert d.year == 2024
        assert d.month == 3
        assert d.day == 15

    def test_from_ymd_invalid_raises(self) -> None:
        from pyratatui import CalendarDate

        with pytest.raises(ValueError):
            CalendarDate.from_ymd(2024, 2, 30)
        with pytest.raises(ValueError):
            CalendarDate.from_ymd(2024, 13, 1)

    def test_repr(self) -> None:
        from pyratatui import CalendarDate

        d = CalendarDate.from_ymd(2024, 3, 15)
        assert "2024" in repr(d)
        assert "15" in repr(d)

    def test_str(self) -> None:
        from pyratatui import CalendarDate

        d = CalendarDate.from_ymd(2024, 3, 15)
        s = str(d)
        assert "2024" in s and "15" in s

    def test_equality(self) -> None:
        from pyratatui import CalendarDate

        a = CalendarDate.from_ymd(2024, 1, 1)
        b = CalendarDate.from_ymd(2024, 1, 1)
        c = CalendarDate.from_ymd(2024, 1, 2)
        assert a == b
        assert a != c

    def test_hashable(self) -> None:
        from pyratatui import CalendarDate

        d1 = CalendarDate.from_ymd(2024, 6, 15)
        d2 = CalendarDate.from_ymd(2024, 6, 15)
        s = {d1, d2}
        assert len(s) == 1
        m = {d1: "event"}
        assert m[d2] == "event"


class TestCalendarEventStore:
    def test_new(self) -> None:
        from pyratatui import CalendarEventStore

        store = CalendarEventStore()
        assert store is not None

    def test_add(self) -> None:
        from pyratatui import CalendarDate, CalendarEventStore, Color, Style

        store = CalendarEventStore()
        store.add(CalendarDate.from_ymd(2024, 12, 25), Style().fg(Color.red()).bold())
        assert "1" in repr(store)

    def test_add_today(self) -> None:
        from pyratatui import CalendarEventStore, Color, Style

        store = CalendarEventStore()
        store.add_today(Style().fg(Color.green()))
        assert store is not None

    def test_today_highlighted(self) -> None:
        from pyratatui import CalendarEventStore, Color, Style

        store = CalendarEventStore.today_highlighted(Style().fg(Color.cyan()).bold())
        assert store is not None
        assert "1" in repr(store)

    def test_repr(self) -> None:
        from pyratatui import CalendarEventStore

        r = repr(CalendarEventStore())
        assert "CalendarEventStore" in r


class TestMonthly:
    def test_creation(self) -> None:
        from pyratatui import CalendarDate, CalendarEventStore, Monthly

        d = CalendarDate.from_ymd(2024, 3, 1)
        store = CalendarEventStore()
        cal = Monthly(d, store)
        assert cal is not None

    def test_builder_methods(self) -> None:
        from pyratatui import (
            Block,
            CalendarDate,
            CalendarEventStore,
            Color,
            Monthly,
            Style,
        )

        d = CalendarDate.from_ymd(2024, 6, 1)
        store = CalendarEventStore()
        cal = (
            Monthly(d, store)
            .block(Block().bordered().title(" June "))
            .show_month_header(Style().bold().fg(Color.cyan()))
            .show_weekdays_header(Style().italic())
            .show_surrounding(Style().dim())
            .default_style(Style().fg(Color.white()))
        )
        assert cal is not None

    def test_repr(self) -> None:
        from pyratatui import CalendarDate, CalendarEventStore, Monthly

        cal = Monthly(CalendarDate.from_ymd(2024, 3, 1), CalendarEventStore())
        r = repr(cal)
        assert "Monthly" in r
        assert "2024" in r


class TestAsyncTerminal:
    def test_import(self) -> None:
        from pyratatui import AsyncTerminal

        at = AsyncTerminal()
        assert at is not None
        assert "active=False" in repr(at)

    def test_run_app_import(self) -> None:
        from pyratatui import run_app, run_app_async

        assert callable(run_app)
        assert inspect.iscoroutinefunction(run_app_async)


class TestExceptions:
    def test_hierarchy(self) -> None:
        from pyratatui import (
            AsyncError,
            BackendError,
            LayoutError,
            PyratatuiError,
            RenderError,
            StyleError,
        )

        assert issubclass(BackendError, PyratatuiError)
        assert issubclass(LayoutError, PyratatuiError)
        assert issubclass(RenderError, PyratatuiError)
        assert issubclass(AsyncError, PyratatuiError)
        assert issubclass(StyleError, PyratatuiError)
        assert issubclass(PyratatuiError, Exception)

    def test_raise_and_catch(self) -> None:
        from pyratatui import LayoutError, PyratatuiError

        with pytest.raises(PyratatuiError):
            raise LayoutError("test error")


class TestCanvasWidget:
    """Test Canvas widget."""

    def test_canvas_creation(self) -> None:
        """Test creating a canvas."""
        from pyratatui import Canvas

        canvas = Canvas(100, 50)
        assert canvas is not None

    def test_canvas_draw_point(self) -> None:
        """Test drawing a point."""
        from pyratatui import Canvas

        canvas = Canvas(100, 50)
        canvas.draw_point(10.0, 20.0)
        assert canvas.len == 1

    def test_canvas_draw_line(self) -> None:
        """Test drawing a line."""
        from pyratatui import Canvas

        canvas = Canvas(100, 50)
        canvas.draw_line(0.0, 0.0, 10.0, 10.0)
        assert canvas.len == 1

    def test_canvas_draw_rect(self) -> None:
        """Test drawing a rectangle."""
        from pyratatui import Canvas

        canvas = Canvas(100, 50)
        canvas.draw_rect(10.0, 10.0, 20.0, 20.0)
        assert canvas.len == 1

    def test_canvas_clear(self) -> None:
        """Test clearing canvas."""
        from pyratatui import Canvas

        canvas = Canvas(100, 50)
        canvas.draw_point(10.0, 20.0)
        assert canvas.len == 1
        canvas.clear()
        assert canvas.len == 0


class TestMapWidget:
    """Test Map widget."""

    def test_map_creation(self) -> None:
        """Test creating a map."""
        from pyratatui import Map

        map_widget = Map()
        assert map_widget is not None

    def test_map_resolution(self) -> None:
        """Test map resolution."""
        from pyratatui import Map, MapResolution

        map_widget = Map().resolution(MapResolution.High)
        assert map_widget is not None


class TestChart:
    def test_chart_multiple_datasets(self) -> None:
        from pyratatui import Axis, Chart, Dataset, GraphType, LegendPosition, Marker

        line_data = [(0.0, 1.0), (1.0, 3.0), (2.0, 2.0), (3.0, 4.0)]
        scatter_data = [(0.5, 1.5), (1.5, 2.5), (2.5, 3.0)]
        bar_data = [(0.0, 2.0), (1.0, 1.0), (2.0, 3.5), (3.0, 2.2)]

        datasets = [
            Dataset(line_data).name("Line").graph_type(GraphType.Line).marker(Marker.Braille),
            Dataset(scatter_data).name("Scatter").graph_type(GraphType.Scatter).marker(Marker.Dot),
            Dataset(bar_data).name("Bars").graph_type(GraphType.Bar).marker(Marker.Bar),
        ]

        chart = (
            Chart(datasets)
            .x_axis(Axis().title("X").bounds(0.0, 3.0).labels(["0", "1.5", "3"]))
            .y_axis(Axis().title("Y").bounds(0.0, 5.0).labels(["0", "2.5", "5"]))
            .legend_position(LegendPosition.TopRight)
        )
        assert chart is not None
        assert "Chart" in repr(chart)


class TestVersion:
    def test_versions(self) -> None:
        import pyratatui

        assert pyratatui.__version__ == "0.3.0"
        assert pyratatui.__ratatui_version__ == "0.30.2"

    def test_all_exports_importable(self) -> None:
        import pyratatui

        for name in pyratatui.__all__:
            assert hasattr(pyratatui, name), f"Missing: {name}"

    def test_removed_integrations_are_gone(self) -> None:
        import pyratatui

        removed = [
            "Effect",
            "EffectManager",
            "Interpolation",
            "compile_effect",
            "Popup",
            "PopupState",
            "TextArea",
            "ScrollView",
            "ScrollViewState",
            "QrCodeWidget",
            "BarGraph",
            "Tree",
            "TreeState",
            "TreeItem",
            "markdown_to_text",
            "TuiLoggerWidget",
            "TuiWidgetState",
            "init_logger",
            "log_message",
            "ImagePicker",
            "ImageState",
            "ImageWidget",
            "Throbber",
            "Menu",
            "MenuState",
            "MenuItem",
            "MenuEvent",
            "PieChart",
            "PieData",
            "PieStyle",
            "Checkbox",
            "Button",
            "CrosstermBackend",
        ]
        for name in removed:
            assert not hasattr(pyratatui, name), f"Should be removed: {name}"


class TestKeyEvent:
    def test_importable_by_name(self) -> None:
        from pyratatui import KeyEvent

        assert KeyEvent is not None


class TestBuffer:
    def test_create_and_area(self) -> None:
        from pyratatui import Buffer, Rect

        buf = Buffer(Rect(0, 0, 10, 4))
        assert buf.area == Rect(0, 0, 10, 4)

    def test_set_and_get_string(self) -> None:
        from pyratatui import Buffer, Color, Rect, Style

        buf = Buffer(Rect(0, 0, 10, 4))
        buf.set_string(0, 0, "Hi", Style().fg(Color.red()))
        assert buf.get_string(0, 0, 2) == "Hi"

    def test_reset_and_merge(self) -> None:
        from pyratatui import Buffer, Rect

        a = Buffer(Rect(0, 0, 10, 4))
        b = Buffer(Rect(0, 0, 10, 4))
        b.set_string(0, 0, "Hi", None)
        a.merge(b)
        assert a.get_string(0, 0, 2) == "Hi"
        a.reset()
        assert a.get_string(0, 0, 2) == "  "


class TestPrompts:
    def test_text_state_lifecycle(self) -> None:
        from pyratatui import TextState

        state = TextState()
        assert state.is_pending()
        assert state.value() == ""
        state.focus()
        assert state.is_focused

    def test_text_state_typing_and_complete(self) -> None:
        from pyratatui import TextState

        state = TextState()
        state.focus()

        class Key:
            def __init__(self, code: str, ctrl: bool = False, alt: bool = False) -> None:
                self.code = code
                self.ctrl = ctrl
                self.alt = alt

        assert state.handle_key(Key("h"))
        assert state.handle_key(Key("i"))
        assert state.value() == "hi"
        assert state.cursor_pos == 2
        assert state.handle_key(Key("Enter"))
        assert state.is_complete()

    def test_text_state_abort(self) -> None:
        from pyratatui import TextState

        state = TextState("draft")
        assert state.value() == "draft"

        class Key:
            code = "Esc"
            ctrl = False
            alt = False

        assert state.handle_key(Key())
        assert state.is_aborted()

    def test_text_state_editing_keys(self) -> None:
        from pyratatui import TextState

        state = TextState("abc")
        assert state.cursor_pos == 3

        class Key:
            def __init__(self, code: str, ctrl: bool = False) -> None:
                self.code = code
                self.ctrl = ctrl
                self.alt = False

        state.handle_key(Key("Backspace"))
        assert state.value() == "ab"
        state.handle_key(Key("Left"))
        state.handle_key(Key("Delete"))
        assert state.value() == "a"
        state.handle_key(Key("Home"))
        state.handle_key(Key("z", ctrl=True))
        assert state.value() == "a"

    def test_prompt_widgets(self) -> None:
        from pyratatui import PasswordPrompt, TextPrompt, TextRenderStyle

        prompt = TextPrompt("Name: ")
        assert "Name" in repr(prompt)
        styled = prompt.with_render_style(TextRenderStyle.Password)
        assert styled is not None
        assert "Password" in repr(PasswordPrompt("Token: "))

    def test_prompt_helpers_importable(self) -> None:
        from pyratatui import prompt_password, prompt_text

        assert callable(prompt_text)
        assert callable(prompt_password)

    def test_calendar_and_chart_still_present(self) -> None:
        from pyratatui import (
            Axis,
            CalendarDate,
            Chart,
            Dataset,
            Map,
            MapResolution,
            Monthly,
        )

        assert Map().resolution(MapResolution.High) is not None
        assert CalendarDate.from_ymd(2024, 1, 1).year == 2024
        assert Chart([Dataset([(0.0, 1.0)])]) is not None
        assert Axis().title("x") is not None
        assert Monthly is not None
