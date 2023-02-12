static void HmGui_InitWidget (HmGuiWidget* e, uint32 type) {
  e->parent = self.group;
  e->next = 0;
  e->prev = self.group ? self.group->tail : 0;

  if (e->parent) {
    e->parent->children++;
    e->hash = Hash_FNV64_Incremental(
      e->parent->widget.hash,
      &e->parent->children,
      sizeof(e->parent->children));
    if (e->next) {
      e->next->prev = e;
    } else {
      e->parent->tail = e;
    }
    if (e->prev) {
      e->prev->next = e;
    } else {
      e->parent->head = e;
    }
  } else {
    e->hash = Hash_FNV64_Init();
  }

  e->type = type;
  e->pos = Vec2f_Create(0, 0);
  e->size = Vec2f_Create(0, 0);
  e->minSize = Vec2f_Create(0, 0);
  e->align = Vec2f_Create(0, 0);
  e->stretch = Vec2f_Create(0, 0);

  self.last = e;
}

static void HmGui_BeginGroup (uint32 layout) {
  HmGuiGroup* e = MemNew(HmGuiGroup);
  HmGui_InitWidget(&e->widget, Widget_Group);
  e->head = 0;
  e->tail = 0;
  e->layout = layout;
  e->children = 0;
  e->focusStyle = FocusStyle_None;
  e->paddingLower = Vec2f_Create(0, 0);
  e->paddingUpper = Vec2f_Create(0, 0);
  e->offset = Vec2f_Create(0, 0);
  e->maxSize = Vec2f_Create(1e30f, 1e30f);
  e->spacing = self.style->spacing;
  e->frameOpacity = 0.0f;
  e->clip = false;
  e->expand = true;
  for (int i = 0; i < FocusType_SIZE; ++i) e->focusable[i] = false;
  e->storeSize = false;
  self.group = e;

  switch (layout) {
    case Layout_Stack:
      e->widget.stretch = Vec2f_Create(1, 1);
      break;
    case Layout_Vertical:
      e->widget.stretch = Vec2f_Create(1, 0);
      break;
    case Layout_Horizontal:
      e->widget.stretch = Vec2f_Create(0, 1);
      break;
  }
}

static void HmGui_FreeText (HmGuiText* e) {
  StrFree(e->text);
  MemFree(e);
}

static void HmGui_FreeGroup (HmGuiGroup* g) {
  HmGuiWidget* e = g->head;
  while (e) {
    HmGuiWidget* next = e->next;
    switch (e->type) {
      case Widget_Group: HmGui_FreeGroup((HmGuiGroup*)e); break;
      case Widget_Text: HmGui_FreeText((HmGuiText*)e); break;
      default: MemFree(e); break;
    }
    e = next;
  }
  MemFree(g);
}

/* -------------------------------------------------------------------------- */

static HmGuiData* HmGui_GetData (HmGuiGroup* g) {
  HmGuiData* data = (HmGuiData*)HashMap_GetRaw(self.data, g->widget.hash);
  if (!data) {
    data = MemNew(HmGuiData);
    data->offset = Vec2f_Create(0, 0);
    data->minSize = Vec2f_Create(0, 0);
    data->size = Vec2f_Create(0, 0);
    HashMap_SetRaw(self.data, g->widget.hash, data);
  }
  return data;
}

/* -------------------------------------------------------------------------- */

static void HmGui_PushClipRect (HmGuiGroup* g) {
  HmGuiClipRect* rect = MemNew(HmGuiClipRect);
  rect->prev = self.clipRect;
  rect->lower = g->widget.pos;
  rect->upper = Vec2f_Add(g->widget.pos, g->widget.size);
  if (rect->prev) {
    rect->lower.x = Max(rect->lower.x, rect->prev->lower.x);
    rect->lower.y = Max(rect->lower.y, rect->prev->lower.y);
    rect->upper.x = Min(rect->upper.x, rect->prev->upper.x);
    rect->upper.y = Min(rect->upper.y, rect->prev->upper.y);
  }
  self.clipRect = rect;
}

static void HmGui_PopClipRect () {
  HmGuiClipRect* rect = self.clipRect;
  self.clipRect = rect->prev;
  MemFree(rect);
}

/* -------------------------------------------------------------------------- */

static void HmGui_ComputeSize (HmGuiGroup* g) {
  for (HmGuiWidget* e = g->head; e; e = e->next)
    if (e->type == Widget_Group)
      HmGui_ComputeSize((HmGuiGroup*)e);

  g->widget.minSize = Vec2f_Create(0, 0);

  for (HmGuiWidget* e = g->head; e; e = e->next) {
    switch (g->layout) {
      case Layout_Stack:
        g->widget.minSize.x = Max(g->widget.minSize.x, e->minSize.x);
        g->widget.minSize.y = Max(g->widget.minSize.y, e->minSize.y);
        break;
      case Layout_Vertical:
        g->widget.minSize.x  = Max(g->widget.minSize.x, e->minSize.x);
        g->widget.minSize.y += e->minSize.y;
        if (e != g->head) g->widget.minSize.y += g->spacing;
        break;
      case Layout_Horizontal:
        g->widget.minSize.x += e->minSize.x;
        g->widget.minSize.y  = Max(g->widget.minSize.y, e->minSize.y);
        if (e != g->head) g->widget.minSize.x += g->spacing;
        break;
    }
  }

  g->widget.minSize.x += g->paddingLower.x + g->paddingUpper.x;
  g->widget.minSize.y += g->paddingLower.y + g->paddingUpper.y;

  if (g->storeSize) {
    HmGuiData* data = HmGui_GetData(g);
    data->minSize = g->widget.minSize;
  }

  g->widget.minSize.x = Min(g->widget.minSize.x, g->maxSize.x);
  g->widget.minSize.y = Min(g->widget.minSize.y, g->maxSize.y);
}

static void HmGui_LayoutWidget (HmGuiWidget* e, Vec2f pos, float sx, float sy) {
  e->pos = pos;
  e->size = e->minSize;
  e->size.x += e->stretch.x * (sx - e->minSize.x);
  e->size.y += e->stretch.y * (sy - e->minSize.y);
  e->pos.x += e->align.x * (sx - e->size.x);
  e->pos.y += e->align.y * (sy - e->size.y);
}

static void HmGui_LayoutGroup (HmGuiGroup* g) {
  Vec2f pos = g->widget.pos;
  Vec2f size = g->widget.size;
  float extra = 0;
  float totalStretch = 0;

  pos.x += g->paddingLower.x + g->offset.x;
  pos.y += g->paddingLower.y + g->offset.y;
  size.x -= g->paddingLower.x + g->paddingUpper.x;
  size.y -= g->paddingLower.y + g->paddingUpper.y;

  if (g->expand) {
    if (g->layout == Layout_Vertical) {
      extra = g->widget.size.y - g->widget.minSize.y;
      for (HmGuiWidget* e = g->head; e; e = e->next)
        totalStretch += e->stretch.y;
    } else if (g->layout == Layout_Horizontal) {
      extra = g->widget.size.x - g->widget.minSize.x;
      for (HmGuiWidget* e = g->head; e; e = e->next)
        totalStretch += e->stretch.x;
    }

    if (totalStretch > 0)
      extra /= totalStretch;
  }

  float s;
  for (HmGuiWidget* e = g->head; e; e = e->next) {
    switch (g->layout) {
      case Layout_None:
        HmGui_LayoutWidget(e, e->pos, size.x, size.y);
        break;
      case Layout_Stack:
        HmGui_LayoutWidget(e, pos, size.x, size.y);
        break;
      case Layout_Vertical:
        s = e->minSize.y;
        if (extra > 0) s += e->stretch.y * extra;
        HmGui_LayoutWidget(e, pos, size.x, s);
        pos.y += e->size.y + g->spacing;
        break;
      case Layout_Horizontal:
        s = e->minSize.x;
        if (extra > 0) s += e->stretch.x * extra;
        HmGui_LayoutWidget(e, pos, s, size.y);
        pos.x += e->size.x + g->spacing;
        break;
    }

    if (e->type == Widget_Group)
      HmGui_LayoutGroup((HmGuiGroup*)e);
  }


  if (g->storeSize) {
    HmGuiData* data = HmGui_GetData(g);
    data->size = g->widget.size;
  }
}

inline static bool IsClipped (HmGuiGroup* g, Vec2f p) {
  return
    p.x < g->widget.pos.x ||
    p.y < g->widget.pos.y ||
    g->widget.pos.x + g->widget.size.x < p.x ||
    g->widget.pos.y + g->widget.size.y < p.y;
}

static void HmGui_CheckFocus (HmGuiGroup* g) {
  if (g->clip && IsClipped(g, self.focusPos))
    return;

  for (HmGuiWidget* e = g->tail; e; e = e->prev)
    if (e->type == Widget_Group)
      HmGui_CheckFocus((HmGuiGroup*)e);

  for (int i = 0; i < FocusType_SIZE; ++i) {
    if (self.focus[i] == 0 && g->focusable[i]) {
      if (g->widget.pos.x <= self.focusPos.x &&
          g->widget.pos.y <= self.focusPos.y &&
          self.focusPos.x <= g->widget.pos.x + g->widget.size.x &&
          self.focusPos.y <= g->widget.pos.y + g->widget.size.y)
      {
        self.focus[i] = g->widget.hash;
      }
    }
  }
}

/* -------------------------------------------------------------------------- */

static void HmGui_DrawText (HmGuiText* e) {
#if HMGUI_DRAW_GROUP_FRAMES
  Draw_Color(0.5f, 0.2f, 0.2f, 0.5f);
  Draw_Border(1.0f, e->pos.x, e->pos.y, e->size.x, e->size.y);
#endif

  UIRenderer_Text(e->font, e->text, e->widget.pos.x, e->widget.pos.y + e->widget.minSize.y, UNPACK4(e->color));
}

static void HmGui_DrawRect (HmGuiRect* e) {
  UIRenderer_Rect(e->widget.pos.x, e->widget.pos.y, e->widget.size.x, e->widget.size.y, UNPACK4(e->color), false);
}

static void HmGui_DrawImage (HmGuiImage* e) {
  UIRenderer_Image(e->image, e->widget.pos.x, e->widget.pos.y, e->widget.size.x, e->widget.size.y);
}

static void HmGui_DrawGroup (HmGuiGroup* g) {
#if HMGUI_DRAW_GROUP_FRAMES
  Draw_Color(0.2f, 0.2f, 0.2f, 0.5f);
  Draw_Border(2.0f, g->widget.pos.x, g->widget.pos.y, g->widget.size.x, g->widget.size.y);
#endif

  UIRenderer_BeginLayer(g->widget.pos.x, g->widget.pos.y, g->widget.size.x, g->widget.size.y, g->clip);

  for (HmGuiWidget* e = g->tail; e; e = e->prev) {
    switch (e->type) {
      case Widget_Group: HmGui_DrawGroup((HmGuiGroup*)e); break;
      case Widget_Text: HmGui_DrawText((HmGuiText*)e); break;
      case Widget_Rect: HmGui_DrawRect((HmGuiRect*)e); break;
      case Widget_Image: HmGui_DrawImage((HmGuiImage*)e); break;
    }
  }

  if (g->focusable[FocusType_Mouse]) {
    bool focus = self.focus[FocusType_Mouse] == g->widget.hash;
    if (g->focusStyle == FocusStyle_None) {
      UIRenderer_Panel(g->widget.pos.x, g->widget.pos.y, g->widget.size.x, g->widget.size.y, 0.1f, 0.12f, 0.13f, 1.0f, 8.0f, g->frameOpacity);
    }

    else if (g->focusStyle == FocusStyle_Fill) {
      if (focus)
        UIRenderer_Panel(g->widget.pos.x, g->widget.pos.y, g->widget.size.x, g->widget.size.y, 0.1f, 0.5f, 1.0f, 1.0f, 0.0f, 1.0f);
      else
        UIRenderer_Panel(g->widget.pos.x, g->widget.pos.y, g->widget.size.x, g->widget.size.y, 0.15f, 0.15f, 0.15f, 0.8f, 0.0f, g->frameOpacity);
    }

    else if (g->focusStyle == FocusStyle_Outline) {
      if (focus) {
        UIRenderer_Rect(g->widget.pos.x, g->widget.pos.y, g->widget.size.x, g->widget.size.y, 0.1f, 0.5f, 1.0f, 1.0f, true);
      }
    }

    else if (g->focusStyle == FocusStyle_Underline) {
      UIRenderer_Rect(g->widget.pos.x, g->widget.pos.y, g->widget.size.x, g->widget.size.y, 0.3f, 0.3f, 0.3f, focus ? 0.5f : g->frameOpacity, false);
    }
  }

  UIRenderer_EndLayer();
}

/* -------------------------------------------------------------------------- */
