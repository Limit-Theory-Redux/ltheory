function renderAllEntities()
    renderState = RenderState(eye, ..)
    for entity in entities
      local rc = entity.getRenderComponent()
      for autoVar in rc.autoShaderVars
        autoVar.updateShaderVar(renderState, entity) -- This calls the callback function
      end
      rc.drawMesh()
    end
  end