require "json"

class ObjectifiedHash
  def initialize(hash)
    @data = build(hash)
  end

  def method_missing(key, *args)
    if respond_to_missing?(key)
      if /^.*=$/.match?(key)
        raise FrozenError, "can't modify frozen object" if frozen?
        @data[key[0..-2].to_sym] = args.first
      else
        @data.fetch(key, nil)
      end
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

  def freeze
    @data.freeze
    self
  end

  def frozen?
    @data.frozen?
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
    return true if /^.*=$/.match?(name)
    @data.keys.include?(name) || super
  end
end
