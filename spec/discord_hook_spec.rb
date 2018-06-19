require "spec_helper"
require "discord_hooks"

RSpec.describe DiscordHooks do
  context "push hook" do
    let(:gl_push) { objectified_fixture("gl_push.json") }
    let(:expected) do
      desc = <<~COMMITS
        [`679ac842`](https://gitlab.com/testmaster/project/commit/679ac842ad4e77a9) More fixes - **Testmaster**
        [`45280848`](https://gitlab.com/testmaster/project/commit/4528084858866822) Fixed stuff - **Testmaster**
        [`89e617d5`](https://gitlab.com/testmaster/project/commit/89e617d5b82ee14c) Why is everything broken? - **Testmaster**
        [`df9eb970`](https://gitlab.com/testmaster/project/commit/df9eb9704fa4cf59) Added some stuff - **Testmaster**
        [`244a1db7`](https://gitlab.com/testmaster/project/commit/244a1db7f5de8052) Witty commit message - **Testmaster**
      COMMITS
      {
        author: { name: "Testmaster", icon_url: "http://example.com/testmaster.png" },
        title: "Project - 5 new commits",
        url: "https://gitlab.com/testmaster/project",
        description: desc.chomp,
        color: 0xFC6D26,
        footer: { text: "testmaster/project", icon_url: "https://gitlab.com/testmaster/project/avatar.png" }
      }
    end

    it "creates an embed on push" do
      expect(described_class.push_hook(gl_push)).to eq(expected)
    end
  end
end
