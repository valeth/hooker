require "json"

class ObjectifiedHash
  def initialize(hash)
    @data = build(hash)
  end

  def method_missing(key)
    if respond_to_missing?(key)
      @data.fetch(key.to_sym, nil)
    else
      super
    end
  end

  def to_h
    @data.each_with_object({}) do |(key, value), acc|
      acc[key] =
        case value
        when self.class then value.to_h
        when Array then value.map { |v| v.is_a?(self.class) ? v.to_h : v }
        else value
        end
    end
  end

  def to_json
    to_h.to_json
  end

  def ==(other)
    return false unless other.is_a?(self.class)
    to_h == other.to_h
  end

private

  def build(hash)
    hash.each_with_object({}) do |(key, value), acc|
      acc[key.to_sym] =
        case value
        when Hash then self.class.new(value)
        when Array then value.map { |v| v.is_a?(Hash) ? self.class.new(v) : v }
        else value
        end
    end
  end

  def respond_to_missing?(name, include_private = false)
    @data.keys.include?(name.to_sym) || super
  end
end
