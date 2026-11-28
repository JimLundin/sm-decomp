from hypothesis import settings

settings.register_profile("fast", max_examples=10)
settings.register_profile("full", max_examples=1000)
settings.register_profile(
    "mutation", max_examples=10, deadline=None, derandomize=True
)
