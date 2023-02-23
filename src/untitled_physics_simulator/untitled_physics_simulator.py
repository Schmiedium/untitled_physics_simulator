class PSComponent:
    def attach_to_entity(self, entity):
        entity.add_component(self)
