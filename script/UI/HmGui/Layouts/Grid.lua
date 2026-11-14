local Grid = {}
Grid.__index = Grid

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class UILayout
---@field visible boolean
---@field align table<AlignHorizontal, AlignVertical>
---@field padding table<number, number>
---@field margin table<number, number>
---@field layoutType GuiLayoutType
---@field contents table<UIComponent>
---@field widthInLayout number
---@field heightInLayout number
---@field color UIGridColors
---@field showGrid boolean

---@class UILayoutGrid: UILayout

---@class UILayoutGridConstructor: UILayout
---@field visible boolean
---@field align table<AlignHorizontal, AlignVertical>
---@field childrenAlign table<AlignHorizontal, AlignVertical>|nil where cells that don't fill the grid sit in it (like `UIComponent.Container`'s `childrenAlign`)
---@field cellAlign table<AlignHorizontal, AlignVertical>|nil when set, every child keeps its own size and sits in its cell at this alignment
---@field padding table<number, number> `{ x, y }` or `{ left, top, right, bottom }`
---@field margin table<number, number>
---@field spacing number|nil pixels between cells
---@field fixedWidth number|nil width of the grid in pixels (default: from `align` and the contents)
---@field fixedHeight number|nil height of the grid in pixels (default: from `align` and the contents)
---@field layoutType GuiLayoutType Horizontal (cells side by side, the default) or Vertical (cells top to bottom)
---@field cells UILayoutGridCell|table<UILayoutGridCell>|nil the cells' sizes along the layout direction: one for every cell, or a list with one per child (see below)
---@field contents table<UIComponent>
---@field widthInLayout number
---@field heightInLayout number
---@field color UIGridColors
---@field showGrid boolean|nil

---@class UIGridColors
---@field background Color|nil

---@class UILayoutGridCell
---@field kind string "fraction", "fixed" or "fit"
---@field value number|fun(): number|nil

---A cell that takes this share of the grid along the layout direction, like a child's
---`widthInLayout` / `heightInLayout`.
---@param fraction number|fun(): number share of the grid, 0 to 1
---@return UILayoutGridCell
function Grid.Fraction(fraction)
    return { kind = "fraction", value = fraction }
end

---A cell this many pixels wide (horizontal grid) or high (vertical grid).
---@param pixels number|fun(): number
---@return UILayoutGridCell
function Grid.Fixed(pixels)
    return { kind = "fixed", value = pixels }
end

---A cell as wide (horizontal grid) or high (vertical grid) as its content.
---@return UILayoutGridCell
function Grid.Fit()
    return { kind = "fit" }
end

---@param value any|fun(): any
---@return any
local function resolve(value)
    if type(value) == "function" then
        return value()
    end
    return value
end

---@param padding table<number> `{ x, y }` or `{ left, top, right, bottom }`
local function setPadding(padding)
    if #padding == 4 then
        Gui:setPaddingEx(padding[1], padding[2], padding[3], padding[4])
    else
        Gui:setPadding(padding[1], padding[2])
    end
end

---The size of a cell along the layout direction: its entry in `cells`, else the child's
---own `widthInLayout` / `heightInLayout`, else an even share of the grid.
---@param cells UILayoutGridCell|table<UILayoutGridCell>|nil the grid's `cells`
---@return string kind "fraction", "fixed" or "fit"
---@return number|nil value percent of the grid for "fraction", pixels for "fixed"
local function cellSize(cells, horizontal, index, count, child)
    local cell = cells and (cells.kind and cells or cells[index])

    if cell then
        if cell.kind == "fraction" then
            return "fraction", resolve(cell.value) * 100
        end
        return cell.kind, resolve(cell.value)
    end

    local share = child and child.state and (horizontal and child.state.widthInLayout or child.state.heightInLayout)

    if share then
        return "fraction", share() * 100
    end
    return "fraction", 100 / count -- even distribution
end

---Sizes the cell that was just ended. A cell always fills the grid across the layout
---direction.
local function setCellSize(horizontal, kind, value)
    if kind == "fixed" then
        if horizontal then
            Gui:setFixedWidth(value)
            Gui:setPercentHeight(100)
        else
            Gui:setPercentWidth(100)
            Gui:setFixedHeight(value)
        end
    elseif kind == "fit" then
        if horizontal then
            Gui:setPercentHeight(100)
        else
            Gui:setPercentWidth(100)
        end
    elseif horizontal then
        Gui:setPercentSize(value, 100)
    else
        Gui:setPercentSize(100, value)
    end
end

---returns a grid layout object: its children side by side (Horizontal) or top to bottom
---(Vertical), each in a cell of its own.
---
---Cell sizes along the layout direction come from `cells`, either one size for every cell
---or a list with one per child, made with:
--- - `UILayout.Grid.Fraction(f)`: a share of the grid (0 to 1)
--- - `UILayout.Grid.Fixed(px)`: a size in pixels
--- - `UILayout.Grid.Fit()`: as big as the cell's content
---
---A child without an entry in `cells` takes its own `widthInLayout` / `heightInLayout`
---as a fraction, or else an even share of the grid. Across the layout direction every
---cell fills the grid, so the cells of a row share its height (and the cells of a column
---its width). Fixed and fit cells that leave space over are placed by `childrenAlign`.
---
---Rows of a table: a vertical grid of horizontal grids that use the same `cells`.
---@param args UILayoutGridConstructor
---@return UILayoutGrid|nil
function Grid:new(args)
    if not args then
        return
    end

    local newGridLayout = {}
    newGridLayout.state = UICore.ComponentState {
        visible = args.visible,
        align = args.align or { AlignHorizontal.Default, AlignVertical.Default },
        childrenAlign = args.childrenAlign,
        cellAlign = args.cellAlign,
        padding = args.padding or { 0, 0 },
        margin = args.margin or { 0, 0 },
        spacing = args.spacing,
        fixedWidth = args.fixedWidth,
        fixedHeight = args.fixedHeight,
        layoutType = args.layoutType or GuiLayoutType.Horizontal,
        cells = args.cells,
        contents = args.contents,
        widthInLayout = args.widthInLayout,
        heightInLayout = args.heightInLayout,
        color = {
            background = args.color and args.color.background
        },
        showGrid = args.showGrid or function() return GameState.debug.metricsEnabled end,
        showGridColor = Color((math.random() + math.random(50, 99)) / 100, (math.random() + math.random(50, 99)) / 100, (math.random() + math.random(50, 99)) / 100, .4)
    }

    ---Renders one child in a cell of its own and sizes the cell.
    newGridLayout.renderCell = function(self, cells, horizontal, index, count, child)
        Gui:beginStackContainer()

        if self.state.showGrid() then
            Gui:setAlignment(horizontal and AlignHorizontal.Left or AlignHorizontal.Right, AlignVertical.Top)
            Gui:setBorderWidth(2)
            Gui:setBorderColor(self.state.showGridColor())

            Gui:text(tostring(index), Cache.Font("Exo2", 14), self.state.showGridColor())
        end

        if type(child) == "function" then
            child = child() -- dynamic components
        end

        if child then
            local cellAlign = self.state.cellAlign and self.state.cellAlign()

            if cellAlign then
                -- a box as big as the child, placed in the cell: also lines up components
                -- that set their own alignment
                Gui:beginContainer(GuiLayoutType.Horizontal)
                Gui:setAlignment(cellAlign[1], cellAlign[2])
                child:render()
                Gui:endContainer()
            else
                child:render()
            end
        end

        Gui:endContainer()

        setCellSize(horizontal, cellSize(cells, horizontal, index, count, child))
    end

    newGridLayout.render = function(self)
        local horizontal = self.state.layoutType() == GuiLayoutType.Horizontal

        Gui:beginContainer(self.state.layoutType())
        Gui:setAlignment(self.state.align()[1], self.state.align()[2])
        setPadding(self.state.padding())
        Gui:setMargin(self.state.margin()[1], self.state.margin()[2])

        if self.state.childrenAlign then
            Gui:setChildrenAlignment(self.state.childrenAlign()[1], self.state.childrenAlign()[2])
        end

        if self.state.spacing then
            Gui:setSpacing(self.state.spacing())
        end

        if self.state.fixedWidth then
            Gui:setFixedWidth(self.state.fixedWidth())
        end

        if self.state.fixedHeight then
            Gui:setFixedHeight(self.state.fixedHeight())
        end

        if self.state.color().background then
            Gui:setBackgroundColor(self.state.color().background)
        end

        local cells = self.state.cells and self.state.cells()
        local contents = self.state.contents()

        for index, child in ipairs(contents) do
            self:renderCell(cells, horizontal, index, #contents, child)
        end

        Gui:endContainer()
    end

    return newGridLayout
end

setmetatable(Grid, meta)

-- Add to global UIComponent table
---@type UILayoutGridConstructor
UILayout.Grid = Grid

return Grid
