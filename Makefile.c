# UnEgg - EGG archive decompressor
# Makefile for Linux (glibc or musl)

CXX      ?= g++
CC       ?= gcc
CXXFLAGS ?= -O2 -Wall -Wno-unused -Wno-multichar -std=c++11 -D_7ZIP_ST
CFLAGS   ?= -O2 -Wall -Wno-unused -Wno-multichar -D_7ZIP_ST
LDFLAGS  ?=

PREFIX   ?= /usr/local
SRCDIR    = Source/unegg
BUILDDIR  = build
TARGET    = unegg

# Include paths
INCLUDES  = -I$(SRCDIR)

# Collect sources
CPP_SRCS  = $(shell find $(SRCDIR) -name '*.cpp')
C_SRCS    = $(shell find $(SRCDIR) -name '*.c')

# Object files in build dir
CPP_OBJS  = $(patsubst $(SRCDIR)/%.cpp,$(BUILDDIR)/%.o,$(CPP_SRCS))
C_OBJS    = $(patsubst $(SRCDIR)/%.c,$(BUILDDIR)/%.o,$(C_SRCS))
OBJS      = $(CPP_OBJS) $(C_OBJS)

# Dependency files
CPP_DEPS  = $(CPP_OBJS:.o=.d)
C_DEPS    = $(C_OBJS:.o=.d)

.PHONY: all clean install uninstall

all: $(TARGET)

$(TARGET): $(OBJS)
	@echo "LINK $@"
	$(CXX) $(CXXFLAGS) -o $@ $^ $(LDFLAGS)
	@echo "Build complete: $(TARGET)"

$(BUILDDIR)/%.o: $(SRCDIR)/%.cpp
	@mkdir -p $(dir $@)
	@echo "CXX  $<"
	$(CXX) $(CXXFLAGS) $(INCLUDES) -MMD -MP -c -o $@ $<

$(BUILDDIR)/%.o: $(SRCDIR)/%.c
	@mkdir -p $(dir $@)
	@echo "CC   $<"
	$(CC) $(CFLAGS) $(INCLUDES) -MMD -MP -c -o $@ $<

clean:
	rm -rf $(BUILDDIR) $(TARGET)

install: $(TARGET)
	install -d $(DESTDIR)$(PREFIX)/bin
	install -m 755 $(TARGET) $(DESTDIR)$(PREFIX)/bin/

uninstall:
	rm -f $(DESTDIR)$(PREFIX)/bin/$(TARGET)

-include $(CPP_DEPS) $(C_DEPS)
